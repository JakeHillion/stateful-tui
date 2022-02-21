use log::{info, trace};

use crate::{Component, Context, Error, Props};

use std::io::{self, Write};
use std::pin::Pin;
use std::sync::mpsc::{sync_channel, Receiver as SyncReceiver};
use std::sync::Arc;

use async_std::channel::{self as async_channel, Receiver, Sender};
use async_std::task;
use futures::stream::{select, FuturesOrdered, StreamExt};
use futures::{try_join, Future};

use crossterm::{
    event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers},
    terminal::{self, disable_raw_mode, enable_raw_mode, size},
    ExecutableCommand,
};

pub enum Event {
    /// Request passed up to redraw the component tree
    Redraw,

    /// The terminal has been resized
    Resized(u16, u16),

    /// An effect has been called
    NewEffect(Pin<Box<dyn Future<Output = ()> + Send>>),

    /// Terminate the TUI
    Exit,
}

pub struct Tui {
    terminal: Arc<dyn Fn() -> Box<dyn Write + Send> + Send + Sync>,

    raw: bool,
    alternate_screen: bool,
}

impl Tui {
    /// Spawn a TUI with external props
    pub fn spawn_with_props<T: Props>(root: Box<dyn Component<T>>, props: T) -> Result<(), Error> {
        let mut tui = Self::new_default();
        tui.setup()?;

        task::block_on(tui.render_loop(root, props))?;
        Ok(())
    }

    /// Create a default instance on stdout
    pub fn new_default() -> Tui {
        Tui {
            terminal: Arc::new(|| Box::new(io::stdout())),

            raw: true,
            alternate_screen: true,
        }
    }

    /// Setup the instance
    pub fn setup(&mut self) -> Result<(), Error> {
        let mut terminal = (self.terminal)();

        if self.raw {
            enable_raw_mode()?;
        }
        if self.alternate_screen {
            terminal.execute(terminal::EnterAlternateScreen)?;
        }

        Ok(())
    }

    pub async fn render_loop<T: Props>(
        &mut self,
        root: Box<dyn Component<T>>,
        props: T,
    ) -> Result<(), Error> {
        let (tx_sync, rx_sync) = sync_channel(32);
        let (tx_async, rx_async) = async_channel::unbounded();

        // let rx_sync = Mutex::new(rx_sync);
        let channel_syncer = task::spawn({
            let tx_async = tx_async.clone();
            async move {
                sync_to_async_channel(rx_sync, tx_async).await;
                Ok::<(), Error>(())
            }
        });

        let (tx_effect, rx_effect) = async_channel::unbounded();
        let effects_loop = task::spawn(Self::effects_loop(rx_effect));

        let events_loop = task::spawn(Self::event_loop(tx_async));

        let render_loop = {
            let root_ctx = Context::new(root, props, tx_sync);
            let terminal = self.terminal.clone();

            task::spawn(async move {
                let (width, height) = size()?;
                root_ctx.lock().unwrap().update_size(0..width, 0..height);

                let mut terminal = terminal();
                while let Ok(e) = rx_async.recv().await {
                    match e {
                        Event::Exit => break,
                        Event::Redraw => {
                            root_ctx.lock().unwrap().render(&mut terminal)?;
                            terminal.flush()?;
                        }
                        Event::Resized(x, y) => root_ctx.lock().unwrap().update_size(0..x, 0..y),
                        Event::NewEffect(e) => {
                            trace!("received a new effect to poll, forwarding to async...");
                            tx_effect.send(e).await.unwrap();
                            trace!("finished forwarding new effect to async");
                        }
                    }

                    task::yield_now().await;
                }

                Ok::<(), Error>(())
            })
        };

        try_join!(channel_syncer, render_loop, effects_loop, events_loop)?;

        info!("exiting TUI...");
        Ok(())
    }

    async fn event_loop(tx: Sender<Event>) -> Result<(), Error> {
        loop {
            let e = match event::read()? {
                CrosstermEvent::Resize(width, height) => Some(Event::Resized(width, height)),
                CrosstermEvent::Key(k) => {
                    if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('c') {
                        info!("received Ctrl+C");
                        Some(Event::Exit)
                    } else {
                        None
                    }
                }
                CrosstermEvent::Mouse(_) => None,
            };

            if let Some(e) = e {
                if tx.send(e).await.is_err() {
                    // receiver closed
                    break;
                }
            }
        }

        Ok(())
    }

    async fn effects_loop(
        rx: Receiver<Pin<Box<dyn Future<Output = ()> + Send>>>,
    ) -> Result<(), Error> {
        enum ToHandle {
            New(Pin<Box<dyn Future<Output = ()> + Send>>),
            Complete,
        }

        let mut effects = FuturesOrdered::new();
        let mut rx = rx.map(ToHandle::New);

        loop {
            let mut both = select(&mut rx, &mut effects);

            trace!("awaiting async effect");
            if let Some(e) = both.next().await {
                trace!("received async effect");
                match e {
                    ToHandle::New(n) => {
                        trace!("async received a new effect to poll, adding to futures");
                        effects.push(task::spawn(async move {
                            trace!("async effect future polled");
                            n.await;
                            ToHandle::Complete
                        }))
                    }
                    ToHandle::Complete => {
                        trace!("async effect completed");
                    }
                }
            } else {
                break;
            }
        }

        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        if self.raw {
            disable_raw_mode().unwrap();
        }

        if self.alternate_screen {
            let mut terminal = (self.terminal)();
            terminal.execute(terminal::LeaveAlternateScreen).unwrap();
        }
    }
}

async fn sync_to_async_channel<T>(rx: SyncReceiver<T>, tx: Sender<T>) {
    while let Ok(e) = {
        #[allow(clippy::let_and_return)]
        // without this, this fails to compile
        // rx ends up borrowed for the body of the loop and hence must be Sync
        let e = rx.recv();
        e
    } {
        if tx.send(e).await.is_err() {
            break; // async receiver closed
        }
    }
    // nothing more to receive
}

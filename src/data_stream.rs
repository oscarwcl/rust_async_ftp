use pin_project_lite::pin_project;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
#[cfg(feature = "secure")]
use tokio_rustls::client::TlsStream;

#[cfg(not(feature = "secure"))]
pin_project! {
    /// Data Stream used for communications
    #[project = DataStreamProj]
    pub enum DataStream {
        Tcp { #[pin] stream: TcpStream },
    }
}

// pin-project-lite doesn't support cfg on individual enum variants, so we need to duplicate the
// whole enum.
#[cfg(feature = "secure")]
pin_project! {
    /// Data Stream used for communications
    #[project = DataStreamProj]
    pub enum DataStream {
        Tcp { #[pin] stream: TcpStream },
        Ssl { #[pin] stream: TlsStream<TcpStream> },
    }
}

impl DataStream {
    /// Unwrap the stream into TcpStream. This method is only used in secure connection.
    pub fn into_tcp_stream(self) -> TcpStream {
        match self {
            DataStream::Tcp { stream } => stream,
            #[cfg(feature = "secure")]
            DataStream::Ssl { stream } => stream.into_inner().0,
        }
    }

    /// Test if the stream is secured
    pub fn is_ssl(&self) -> bool {
        match self {
            #[cfg(feature = "secure")]
            DataStream::Ssl { .. } => true,
            _ => false,
        }
    }

    /// Returns a reference to the underlying TcpStream.
    pub fn get_ref(&self) -> &TcpStream {
        match self {
            DataStream::Tcp { ref stream } => stream,
            #[cfg(feature = "secure")]
            DataStream::Ssl { ref stream } => stream.get_ref().0,
        }
    }
}

impl AsyncRead for DataStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.project() {
            DataStreamProj::Tcp { stream } => stream.poll_read(cx, buf),
            #[cfg(feature = "secure")]
            DataStreamProj::Ssl { stream } => stream.poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for DataStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.project() {
            DataStreamProj::Tcp { stream } => stream.poll_write(cx, buf),
            #[cfg(feature = "secure")]
            DataStreamProj::Ssl { stream } => stream.poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.project() {
            DataStreamProj::Tcp { stream } => stream.poll_flush(cx),
            #[cfg(feature = "secure")]
            DataStreamProj::Ssl { stream } => stream.poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.project() {
            DataStreamProj::Tcp { stream } => stream.poll_shutdown(cx),
            #[cfg(feature = "secure")]
            DataStreamProj::Ssl { stream } => stream.poll_shutdown(cx),
        }
    }
}

// Copyright 2023 Divy Srivastava <dj.srivastava23@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bytes::{BufMut, Bytes, BytesMut};

use crate::WebSocketError;

macro_rules! repr_u8 {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
      $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
      $(#[$meta])*
      $vis enum $name {
        $($(#[$vmeta])* $vname $(= $val)?,)*
      }

      impl core::convert::TryFrom<u8> for $name {
        type Error = WebSocketError;

        fn try_from(v: u8) -> Result<Self, Self::Error> {
          match v {
            $(x if x == $name::$vname as u8 => Ok($name::$vname),)*
            _ => Err(WebSocketError::InvalidValue),
          }
        }
      }
    }
}

/// Represents a WebSocket frame.
#[derive(Debug)]
pub struct Frame {
    /// Indicates if this is the final frame in a message.
    pub fin: bool,
    /// The opcode of the frame.
    pub opcode: OpCode,
    /// The payload of the frame.
    pub payload: Bytes,
}

const MAX_HEAD_SIZE: usize = 16;

impl Frame {
    /// Creates a new WebSocket `Frame`.
    pub fn new(fin: bool, opcode: OpCode, payload: Bytes) -> Self {
        Self {
            fin,
            opcode,
            payload,
        }
    }

    /// Create a new WebSocket binary `Frame`.
    ///
    /// This is a convenience method for `Frame::new(true, OpCode::Binary, None, payload)`.
    pub fn binary(payload: Bytes) -> Self {
        Self {
            fin: true,
            opcode: OpCode::Binary,
            payload,
        }
    }

    /// Create a new WebSocket close `Frame`.
    ///
    /// This is a convenience method for `Frame::new(true, OpCode::Close, None, payload)`.
    ///
    /// This method does not check if `code` is a valid close code and `reason` is valid UTF-8.
    pub fn close(code: u16, reason: &[u8]) -> Self {
        let mut payload = BytesMut::with_capacity(2 + reason.len());
        payload.put_u16(code);
        payload.put(reason);

        Self {
            fin: true,
            opcode: OpCode::Close,
            payload: payload.freeze(),
        }
    }

    /// Create a new WebSocket close `Frame` with a raw payload.
    ///
    /// This is a convenience method for `Frame::new(true, OpCode::Close, None, payload)`.
    ///
    /// This method does not check if `payload` is valid Close frame payload.
    pub fn close_raw(payload: Bytes) -> Self {
        Self {
            fin: true,
            opcode: OpCode::Close,
            payload,
        }
    }

    /// Create a new WebSocket pong `Frame`.
    ///
    /// This is a convenience method for `Frame::new(true, OpCode::Pong, None, payload)`.
    pub fn pong(payload: Bytes) -> Self {
        Self {
            fin: true,
            opcode: OpCode::Pong,
            payload,
        }
    }

    /// Checks if the frame payload is valid UTF-8.
    pub fn is_utf8(&self) -> bool {
        return std::str::from_utf8(&self.payload).is_ok();
    }

    /// Formats the frame header into the head buffer. Returns the size of the length field.
    ///
    /// # Panics
    ///
    /// This method panics if the head buffer is not at least n-bytes long, where n is the size of the length field (0, 2, 4, or 10)
    pub fn fmt_head(&mut self, buf: &mut BytesMut) {
        buf.put_u8((self.fin as u8) << 7 | (self.opcode as u8));

        let len = self.payload.len();
        if len < 126 {
            buf.put_u8(len as u8);
        } else if len < 65536 {
            buf.put_u8(126);
            buf.put_u16(len as u16);
        } else {
            buf.put_u8(127);
            buf.put_u64(len as u64);
        };
    }

    /// Writes the frame to the buffer and returns a slice of the buffer containing the frame.
    pub fn write(&mut self, buf: &mut BytesMut) {
        let len = self.payload.len();
        if len > buf.remaining_mut() {
            buf.reserve(len + MAX_HEAD_SIZE - buf.remaining_mut());
        }

        self.fmt_head(buf);
        buf.put(&*self.payload);
    }
}

repr_u8! {
    #[repr(u8)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum OpCode {
        Continuation = 0x0,
        Text = 0x1,
        Binary = 0x2,
        Close = 0x8,
        Ping = 0x9,
        Pong = 0xA,
    }
}

#[inline]
pub fn is_control(opcode: OpCode) -> bool {
    matches!(opcode, OpCode::Close | OpCode::Ping | OpCode::Pong)
}

#![cfg(feature = "std")]

use crate::TryNextWithContext;
use std::io::{self, BufRead, BufReader, Read};

impl<R, C> TryNextWithContext<C> for BufReader<R>
where
    R: Read,
{
    type Item = u8;
    type Error = io::Error;

    fn try_next_with_context(&mut self, _ctx: &mut C) -> Result<Option<Self::Item>, Self::Error> {
        let buf = self.fill_buf()?;
        if buf.is_empty() {
            return Ok(None);
        }
        let b = buf[0];
        self.consume(1);
        Ok(Some(b))
    }
}

use std::{collections::HashSet, mem::size_of, process::Output};

pub trait AsBytes {
    type Output: AsRef<[u8]>;

    fn as_bytes(&self) -> Self::Output;

    fn size_hint(&self) -> usize {
        size_of::<Output>()
    }
}

impl AsBytes for u8 {
    type Output = [u8; 1];

    fn as_bytes(&self) -> Self::Output {
        [*self]
    }

    fn size_hint(&self) -> usize {
        1
    }
}

impl<T: AsBytes> AsBytes for HashSet<T> {
    type Output = Vec<u8>;

    fn as_bytes(&self) -> Self::Output {
        let mut bytes = Vec::with_capacity(self.size_hint());
        for elem in self {
            bytes.extend_from_slice(elem.as_bytes().as_ref());
        }
        bytes
    }

    fn size_hint(&self) -> usize {
        self.len() * size_of::<T>()
    }
}

#[macro_export]
macro_rules! packet {
    [$($elem:expr),*] => {
        {
            let mut cap = 0;
            $(
                cap += $elem.size_hint();
            )*
            let mut packet = Vec::with_capacity(cap);
            $(
                packet.extend_from_slice(&$elem.as_bytes().as_ref());
            )*
            packet
        }
    };
}

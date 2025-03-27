/// A munger which XORs a key with some data
#[derive(Clone)]
pub struct Xorcism<Key> {
    // This field is just to suppress compiler complaints;
    // feel free to delete it at any point.
    key: Key,
}

impl<Key: AsRef<[u8]>> Xorcism<Key> {
    /// Create a new Xorcism munger from a key
    ///
    /// Should accept anything which has a cheap conversion to a byte slice.
    pub fn new(key: Key) -> Xorcism<Key> {
        Self { key }
    }

    /// XOR each byte of the input buffer with a byte from the key.
    ///
    /// Note that this is stateful: repeated calls are likely to produce different results,
    /// even with identical inputs.
    pub fn munge_in_place(&mut self, data: &mut [u8]) {
        let key_bytes = self.key.as_ref();
        let mut extended_key = key_bytes.repeat(data.len() / key_bytes.len());
        let c = &key_bytes[0..data.len() % key_bytes.len()];
        extended_key.extend(c);

        extended_key
            .iter()
            .zip(data)
            .for_each(|(key_byte, data_byte)| *data_byte ^= *key_byte);
    }

    /// XOR each byte of the data with a byte from the key.
    ///
    /// Note that this is stateful: repeated calls are likely to produce different results,
    /// even with identical inputs.
    ///
    /// Should accept anything which has a cheap conversion to a byte iterator.
    /// Shouldn't matter whether the byte iterator's values are owned or borrowed.
    pub fn munge<Data>(&mut self, data: Data) -> impl Iterator<Item = u8>
    where
        Data: AsRef<[u8]>,
    {
        let key_bytes = self.key.as_ref();

        struct MyIter<'b, Data> {
            len: usize,
            key: &'b [u8],
            data: Data,
        }

        impl<Data: AsRef<[u8]>> Iterator for MyIter<'_, Data> {
            type Item = u8;

            fn next(&mut self) -> Option<Self::Item> {
                if self.len >= self.data.as_ref().len() {
                    None
                } else {
                    self.len += 1;
                    Some(self.key[self.len - 1] ^ self.data.as_ref()[self.len - 1])
                }
            }
        }

        MyIter {
            len: 0,
            key: key_bytes,
            data,
        }
    }
}

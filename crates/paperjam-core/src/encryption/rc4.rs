/// Minimal RC4 stream cipher implementation for PDF encryption.
pub struct Rc4 {
    s: [u8; 256],
    i: u8,
    j: u8,
}

impl Rc4 {
    pub fn new(key: &[u8]) -> Self {
        let mut s = [0u8; 256];
        for (i, b) in s.iter_mut().enumerate() {
            *b = i as u8;
        }
        let mut j: u8 = 0;
        for i in 0..256 {
            j = j.wrapping_add(s[i]).wrapping_add(key[i % key.len()]);
            s.swap(i, j as usize);
        }
        Rc4 { s, i: 0, j: 0 }
    }

    pub fn process(&mut self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .map(|&byte| {
                self.i = self.i.wrapping_add(1);
                self.j = self.j.wrapping_add(self.s[self.i as usize]);
                self.s.swap(self.i as usize, self.j as usize);
                let k = self.s
                    [(self.s[self.i as usize].wrapping_add(self.s[self.j as usize])) as usize];
                byte ^ k
            })
            .collect()
    }
}

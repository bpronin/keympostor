use fmt::Display;
use std::{fmt, u128};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Bit256([u128; 2]);

impl Bit256 {
    pub fn from_fn<F>(f: F) -> Self
    where
        F: Fn(usize) -> bool,
    {
        let mut inner = [0u128; 2];
        for i in 0..256 {
            if f(i) {
                let (chunk, bit) = (i / 128, i % 128);
                inner[chunk] |= 1u128 << bit;
            }
        }
        Self(inner)
    }

    // /// Creates a Bit256 structure from a [bool; 256] array
    // pub fn from_bools(bools: [bool; 256]) -> Self {
    //     let mut inner = [0u128; 2];
    //     for i in 0..256 {
    //         if bools[i] {
    //             let (chunk, bit) = (i / 128, i % 128);
    //             inner[chunk] |= 1u128 << bit;
    //         }
    //     }
    //     Self(inner)
    // }

    // /// Converts the Bit256 structure to a [bool; 256] array
    // pub fn to_bools(&self) -> [bool; 256] {
    //     let mut bools = [false; 256];
    //     for i in 0..256 {
    //         let (chunk, bit) = (i / 128, i % 128);
    //         bools[i] = (self.0[chunk] >> bit) & 1 != 0;
    //     }
    //     bools
    // }

    /// Sets the bit at the given index to the specified value (true/false)
    pub fn set(&mut self, index: usize, value: bool) {
        assert!(index < 256, "index out of range");
        let (chunk, bit) = (index / 128, index % 128);
        if value {
            self.0[chunk] |= 1u128 << bit;
        } else {
            self.0[chunk] &= !(1u128 << bit);
        }
    }

    /// Gets the value of the bit at the given index (true/false)
    pub fn get(&self, index: usize) -> bool {
        assert!(index < 256, "index out of range");
        let (chunk, bit) = (index / 128, index % 128);
        (self.0[chunk] >> bit) & 1 != 0
    }
}

impl Default for Bit256 {
    fn default() -> Self {
        Self([0, 0])
    }
}

impl Display for Bit256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // for block in (0..4).rev() {
        //     for i in ((block * 64)..((block + 1) * 64)).rev() {
        //         let bit = self.get(i);
        //         write!(f, "{}", if bit { '1' } else { '0' })?;
        //     }
        //     if block != 0 {
        //         writeln!(f)?;
        //     }
        // }

        writeln!(f, "{:064b}", (self.0[1] >> 64) as u64)?;
        writeln!(f, "{:064b}", self.0[1] as u64)?;
        writeln!(f, "{:064b}", (self.0[0] >> 64) as u64)?;
        writeln!(f, "{:064b}", self.0[0] as u64)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_fn() {
        let a = Bit256::from_fn(|i| i % 2 == 0);
        println!("{}", a);
    }

    #[test]
    fn test_display() {
        let mut a = Bit256::default();
        a.set(0, true);
        a.set(64, true);
        a.set(128, true);
        a.set(192, true);
        println!("{}", a);
    }
}
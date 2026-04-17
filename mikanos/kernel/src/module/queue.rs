use core::mem::MaybeUninit;

use crate::module::err::KResult;
use crate::module::err::KError;


pub struct ArrayQueue<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    read_pos: usize,
    write_pos: usize,
    count: usize,
}

impl<T, const N: usize> ArrayQueue<T, N> {
    pub const fn new() -> Self {                                                                                                                                               
        ArrayQueue {                                                                                                                                                           
            data: [const { MaybeUninit::uninit() }; N],                                                                                                                        
            read_pos: 0,                                                                                                                                                       
            write_pos: 0,                                                                                                                                                    
            count: 0,
        }                                                                                                                                                                      
    }
    
    pub fn push(&mut self, value: T) -> KResult<()>  {
        if self.count == N {
            return Err(KError::Full);
        }

        self.data[self.write_pos] = MaybeUninit::new(value);
        self.count += 1;
        self.write_pos += 1;
        if self.write_pos == N {
            self.write_pos = 0;
        }

        Ok(())
    }

    pub fn pop(&mut self) -> KResult<T> {
        if self.count == 0 {
            return Err(KError::Empty);
        }

        let read_val = unsafe {
            self.data[self.read_pos].assume_init_read()
        };
        self.count -= 1;
        self.read_pos += 1;
        if self.read_pos == N {
            self.read_pos = 0;
        }

        Ok(read_val)
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn capacity(&self) -> usize {
        N
    }
}
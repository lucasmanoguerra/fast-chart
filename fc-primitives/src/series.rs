use std::mem::MaybeUninit;

pub struct TimeSeries<T, const N: usize> {
    buffer: Box<[MaybeUninit<T>; N]>,
    head: usize,
    len: usize,
}

impl<T, const N: usize> TimeSeries<T, N> {
    pub fn new() -> Self {
        // Allocate an uninitialized array on the heap via Vec.
        // SAFETY: MaybeUninit<T> does not require initialization.
        // The Vec is leaked (len=0, cap=N) and reinterpreted as a fixed-size Box.
        let mut vec: Vec<MaybeUninit<T>> = Vec::with_capacity(N);
        // Safety: we just allocated with capacity N, and MaybeUninit is valid uninit
        unsafe { vec.set_len(N) };
        let boxed: Box<[MaybeUninit<T>]> = vec.into_boxed_slice();
        // Safety: boxed.len() == N, so the pointer is aligned and sized as [MaybeUninit<T>; N]
        let buffer = unsafe {
            let raw = Box::into_raw(boxed);
            Box::from_raw(raw as *mut [MaybeUninit<T>; N])
        };
        Self {
            buffer,
            head: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) -> Option<T> {
        let overwritten;
        if self.len == N {
            // Buffer full — overwrite oldest in-place (drop old, write new)
            let slot = &mut self.buffer[self.head];
            // Safety: head always points to an initialized slot when len == N
            // Safety: head always points to an initialized slot when len == N.
            // ptr::read moves the value out without dropping; the slot is then
            // overwritten by write(), so no double-drop occurs.
            overwritten = Some(unsafe { (slot.as_ptr() as *const T).read() });
            slot.write(value);
        } else {
            overwritten = None;
            self.buffer[self.head] = MaybeUninit::new(value);
            self.len += 1;
        }
        self.head = (self.head + 1) % N;
        overwritten
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        N
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        // Logical 0 = oldest element
        let start = if self.len < N {
            0
        } else {
            self.head // head points to oldest when full
        };
        let physical = (start + index) % N;
        Some(unsafe { self.buffer[physical].assume_init_ref() })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.len).filter_map(move |i| self.get(i))
    }

    pub fn latest(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            let idx = self.len - 1;
            self.get(idx)
        }
    }

    pub fn drain_latest(&mut self, count: usize) -> Drain<'_, T, N> {
        let to_drain = count.min(self.len);
        Drain {
            series: self,
            remaining: to_drain,
        }
    }
}

pub struct Drain<'a, T, const N: usize> {
    series: &'a mut TimeSeries<T, N>,
    remaining: usize,
}

impl<T, const N: usize> Iterator for Drain<'_, T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 || self.series.len == 0 {
            return None;
        }
        self.remaining -= 1;
        // Compute physical index of the newest element BEFORE decrementing len
        let start = if self.series.len < N {
            0
        } else {
            self.series.head
        };
        let physical = (start + self.series.len - 1) % N;
        self.series.len -= 1;
        Some(unsafe { self.series.buffer[physical].assume_init_read() })
    }
}

impl<T, const N: usize> Drop for TimeSeries<T, N> {
    fn drop(&mut self) {
        // Drop only initialized elements
        for i in 0..self.len {
            let start = if self.len < N {
                0
            } else {
                self.head
            };
            let physical = (start + i) % N;
            unsafe {
                self.buffer[physical].assume_init_drop();
            }
        }
    }
}

impl<T, const N: usize> Default for TimeSeries<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_series() {
        let s: TimeSeries<i32, 10> = TimeSeries::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
        assert_eq!(s.capacity(), 10);
    }

    #[test]
    fn push_single() {
        let mut s: TimeSeries<i32, 10> = TimeSeries::new();
        assert_eq!(s.push(42), None);
        assert_eq!(s.len(), 1);
        assert_eq!(s.latest(), Some(&42));
    }

    #[test]
    fn push_maintains_order() {
        let mut s: TimeSeries<i32, 10> = TimeSeries::new();
        s.push(1);
        s.push(2);
        s.push(3);
        let items: Vec<&i32> = s.iter().collect();
        assert_eq!(items, vec![&1, &2, &3]);
    }

    #[test]
    fn overflow_overwrites_oldest() {
        let mut s: TimeSeries<i32, 3> = TimeSeries::new();
        s.push(1);
        s.push(2);
        s.push(3);
        let evicted = s.push(4);
        assert_eq!(evicted, Some(1));
        let items: Vec<&i32> = s.iter().collect();
        assert_eq!(items, vec![&2, &3, &4]);
    }

    #[test]
    fn get_returns_none_for_out_of_bounds() {
        let mut s: TimeSeries<i32, 10> = TimeSeries::new();
        s.push(1);
        assert!(s.get(1).is_none());
        assert!(s.get(100).is_none());
    }

    #[test]
    fn get_returns_correct_values() {
        let mut s: TimeSeries<i32, 10> = TimeSeries::new();
        s.push(10);
        s.push(20);
        s.push(30);
        assert_eq!(s.get(0), Some(&10));
        assert_eq!(s.get(1), Some(&20));
        assert_eq!(s.get(2), Some(&30));
    }

    #[test]
    fn latest_on_empty() {
        let s: TimeSeries<i32, 10> = TimeSeries::new();
        assert!(s.latest().is_none());
    }

    #[test]
    fn drain_latest_basic() {
        let mut s: TimeSeries<i32, 10> = TimeSeries::new();
        s.push(1);
        s.push(2);
        s.push(3);
        let drained: Vec<i32> = s.drain_latest(2).collect();
        assert_eq!(drained, vec![3, 2]);
        assert_eq!(s.len(), 1);
        assert_eq!(s.latest(), Some(&1));
    }

    #[test]
    fn drain_latest_more_than_available() {
        let mut s: TimeSeries<i32, 10> = TimeSeries::new();
        s.push(1);
        s.push(2);
        let drained: Vec<i32> = s.drain_latest(100).collect();
        assert_eq!(drained, vec![2, 1]);
        assert!(s.is_empty());
    }

    #[test]
    fn drain_latest_on_empty() {
        let mut s: TimeSeries<i32, 10> = TimeSeries::new();
        let drained: Vec<i32> = s.drain_latest(5).collect();
        assert!(drained.is_empty());
    }

    #[test]
    fn many_pushes_after_overflow() {
        let mut s: TimeSeries<i32, 3> = TimeSeries::new();
        for i in 0..100 {
            s.push(i);
        }
        assert_eq!(s.len(), 3);
        let items: Vec<&i32> = s.iter().collect();
        assert_eq!(items, vec![&97, &98, &99]);
    }

    #[test]
    fn iter_on_empty() {
        let s: TimeSeries<i32, 10> = TimeSeries::new();
        assert_eq!(s.iter().count(), 0);
    }

    #[test]
    fn default_is_empty() {
        let s: TimeSeries<i32, 5> = TimeSeries::default();
        assert!(s.is_empty());
    }

    #[test]
    fn push_returns_overwritten_after_exact_capacity() {
        let mut s: TimeSeries<i32, 2> = TimeSeries::new();
        s.push(10);
        s.push(20);
        // Now full. Next push overwrites.
        let evicted = s.push(30);
        assert_eq!(evicted, Some(10));
    }

    #[test]
    fn drop_correctness() {
        // Verify no double-free or leak by running under valgrind via cargo test
        let mut s: TimeSeries<String, 5> = TimeSeries::new();
        s.push("a".to_string());
        s.push("b".to_string());
        s.push("c".to_string());
        drop(s);
    }

    #[test]
    fn drain_returns_newest_first() {
        let mut s: TimeSeries<i32, 10> = TimeSeries::new();
        for i in 0..5 {
            s.push(i);
        }
        let drained: Vec<i32> = s.drain_latest(3).collect();
        assert_eq!(drained, vec![4, 3, 2]);
    }

    #[test]
    fn overflow_after_wraparound() {
        let mut s: TimeSeries<i32, 3> = TimeSeries::new();
        s.push(1);
        s.push(2);
        s.push(3);
        s.push(4); // evicts 1, head wraps to 1
        s.push(5); // evicts 2, head wraps to 2
        s.push(6); // evicts 3, head wraps to 0
        let items: Vec<&i32> = s.iter().collect();
        assert_eq!(items, vec![&4, &5, &6]);
    }

    #[test]
    fn get_after_wraparound() {
        let mut s: TimeSeries<i32, 3> = TimeSeries::new();
        s.push(10);
        s.push(20);
        s.push(30);
        s.push(40);
        // buffer physically: [40, 20, 30], head=1, len=3, oldest=20 at physical 1
        assert_eq!(s.get(0), Some(&20));
        assert_eq!(s.get(1), Some(&30));
        assert_eq!(s.get(2), Some(&40));
    }
}

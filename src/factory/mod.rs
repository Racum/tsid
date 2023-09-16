use std::ops::Add;
use std::time::{Duration, SystemTime};
use rand::RngCore;
use crate::tsid::TSID;

const TIME_BITS: u8 = 42;
const RANDOM_BITS: u8 = 64 - TIME_BITS;
const RANDOM_MASK: u64 = 0x003fffff;
//22 bits
const TSID_EPOCH_MILLIS: u64 = 1577836800000;

#[derive(Debug)]
pub struct TsidFactory {
    // TODO: Consider if all of those can be generic constants
    node_bits: u8,
    counter_bits: u8,
    counter_mask: u64,
    node_mask: u64,
    last_time_value: u128,
    counter: u64,
    node: u32,
}


impl Default for TsidFactory {
    #[doc = "Create default TsidFactory with `node_bits: 0`"]
    fn default() -> Self {
        TsidFactory::with_node_bits(0, 0)
    }
}

///Example
/// ```rust
/// use tsid::TsidFactory;
/// let factory = TsidFactory::with_node_bits(8,1);
///```
impl TsidFactory {
    /// Create a new TsidFactory with default settings
    /// see [`TsidFactory::default`]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_node_bits(node_bits: u8, node: u32) -> Self {
        let counter_bits: u8 = RANDOM_BITS - node_bits;
        let counter_mask = RANDOM_MASK >> node_bits;
        let node_mask = RANDOM_MASK >> counter_bits;

        let mut rng = rand::thread_rng();
        let counter = rng.next_u64() & counter_mask;
        let last_time_value = Self::get_time_millis_in_tsid_epoch();

        Self {
            node_bits,
            counter_bits,
            counter_mask,
            node_mask,
            last_time_value,
            counter,
            node,
        }
    }


    // naive implementation without thread safety
    pub fn create(&mut self) -> TSID {
        let time = self.get_time_and_advance_counter();
        let node_val: u64 = (self.node << self.counter_bits) as u64;
        let time_val: u64 = (time << RANDOM_BITS) as u64;
        let counter_val = self.counter & self.counter_mask;
        let number = time_val | node_val | counter_val;
        TSID::new(number)
    }

    fn get_time_and_advance_counter(&mut self) -> u128 {
        let mut rng = rand::thread_rng();
        let mut time_millis = Self::get_time_millis_in_tsid_epoch();

        if time_millis <= self.last_time_value {
            self.counter += 1;
            if self.counter >> self.counter_bits > 0 {
                //carry
                time_millis += 1;
            }
        } else {
            self.counter = rng.next_u64();
        }
        self.counter = self.counter & self.counter_mask;
        self.last_time_value = time_millis;

        return time_millis;
    }

    fn get_time_millis_in_tsid_epoch() -> u128 {
        let tsid_epoch = SystemTime::UNIX_EPOCH.add(Duration::from_millis(TSID_EPOCH_MILLIS));

        SystemTime::now()
            .duration_since(tsid_epoch)
            .expect("UNIX_EPOCH ias after now(), check Your system time")
            .as_millis()
    }
}

#[cfg(test)]
mod tests {
    use crate::factory::{TIME_BITS, TsidFactory};

    #[test]
    fn builder_should_set_all_masks_for_8node_bits_version() {
        let factory_under_test = TsidFactory::with_node_bits(8, 0);
        println!("{:?}", factory_under_test);

        assert_eq!(8, factory_under_test.node_bits);
        assert_eq!(14, factory_under_test.counter_bits);
        assert_eq!(0x3fff, factory_under_test.counter_mask);
        assert_eq!(0xff, factory_under_test.node_mask);
        assert_eq!(64, TIME_BITS + factory_under_test.counter_bits + factory_under_test.node_bits)
    }

    #[test]
    fn builder_should_set_all_masks_for_0node_bits_version() {
        let factory_under_test = TsidFactory::with_node_bits(0, 0);
        println!("{:?}", factory_under_test);

        assert_eq!(0, factory_under_test.node_bits);
        assert_eq!(22, factory_under_test.counter_bits);
        assert_eq!(0x3fffff, factory_under_test.counter_mask);
        assert_eq!(0x0, factory_under_test.node_mask);
        assert_eq!(64, TIME_BITS + factory_under_test.counter_bits + factory_under_test.node_bits)
    }

    #[test]
    fn create_tsid() {
        let mut factory_under_test = TsidFactory::with_node_bits(8, 1);
        let _tsid = factory_under_test.create();
        println!("{}", _tsid.to_string())
    }
}
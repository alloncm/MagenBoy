use lib_gb::apu::timer::Timer;

#[test]
fn timer_test() {
    let mut timer = Timer::new(512);
    let mut counter = 0;

    for _ in 0..511 {
        counter += 1;
        assert!(timer.cycle() == false);
    }

    counter += 1;
    assert!(timer.cycle() == true);
    assert!(counter == 512)
}

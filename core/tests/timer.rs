use magenboy_core::apu::timer::Timer;


#[test]
fn timer_test(){
    let count = 512;
    let mut timer = Timer::new(count);
    let mut counter = 0;

    for _ in 0..(count/4)-1{
        counter+=1;
        assert!(timer.cycle() == false);
    }

    counter+=1;
    assert!(timer.cycle() == true);
    assert!(counter == 512/4)
}
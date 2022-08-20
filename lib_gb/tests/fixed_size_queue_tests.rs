use lib_gb::utils::fixed_size_queue::FixedSizeQueue;

#[test]
fn test_fifo(){
    let mut fifo = FixedSizeQueue::<u8, 8>::new();
    fifo.push(10);
    fifo.push(22);

    assert_eq!(fifo.len(), 2);
    assert_eq!(fifo[0], 10);
    assert_eq!(fifo[1], 22);

    fifo.remove();
    assert_eq!(fifo.len(), 1);
    assert_eq!(fifo[0], 22);

    fifo[0] = 21;
    assert_eq!(fifo[0], 21);
}

#[test]
fn test_fifo_wrapping_around(){
    let mut fifo = FixedSizeQueue::<u8, 3>::new();

    check_push_and_remove(&mut fifo);
    check_push_and_remove(&mut fifo);
    check_push_and_remove(&mut fifo);

    fifo.push(10);
    fifo.push(22);
    fifo.remove();
    assert_eq!(fifo.len(), 1);
    assert_eq!(fifo[0], 22);

    fifo[0] = 21;
    assert_eq!(fifo[0], 21);
}

fn check_push_and_remove(fifo: &mut FixedSizeQueue<u8, 3>) {
    fifo.push(10);
    fifo.push(22);
    fifo.push(33);

    assert_eq!(fifo.len(), 3);
    assert_eq!(fifo[0], 10);
    assert_eq!(fifo[1], 22);
    assert_eq!(fifo[2], 33);

    assert_eq!(fifo.remove(), 10);
    assert_eq!(fifo.remove(), 22);
    assert_eq!(fifo.remove(), 33);
    assert_eq!(fifo.len(), 0);
}

#[test]
#[should_panic]
fn panic_on_fifo_full(){
    let mut fifo = FixedSizeQueue::<u8, 3>::new();
    fifo.push(1);
    fifo.push(2);
    fifo.push(3);

    //should panic
    fifo.push(1);
}

#[test]
#[should_panic]
fn panic_on_get_fifo_index_out_of_range(){
    let mut fifo = FixedSizeQueue::<u8, 3>::new();
    fifo.push(1);
    fifo.push(2);

    //should panic
    let _ = fifo[2];
}

#[test]
#[should_panic]
fn panic_on_fifo_set_index_out_of_range(){
    let mut fifo = FixedSizeQueue::<u8, 3>::new();
    fifo.push(1);
    fifo.push(2);

    //should panic
    fifo[2] = 4;
}

#[test]
fn fill_fills_the_fifo(){
    let mut fifo = FixedSizeQueue::<u8, 8>::new();
    fifo.push(1);
    fifo.push(1);
    fifo.push(1);

    fifo.remove();
    fifo.remove();
    
    fifo.fill(&[0;8]);

    assert_eq!(fifo.len(), 8);
    for i in 0..8{
        assert_eq!(fifo[i], 0);
    }
}

#[test]
fn fifo_index_check_happyflow(){
    let mut fifo = FixedSizeQueue::<u8, 8>::new();
    for i in 0..8{
        fifo.push(i);
    }
    for _ in 0..6{
        fifo.remove();
    }
    for i in 0..6{
        fifo.push(i);
    }

    assert_eq!(fifo[0], 6);
    assert_eq!(fifo[1], 7);
    assert_eq!(fifo[2], 0);
    assert_eq!(fifo[3], 1);
    assert_eq!(fifo[4], 2);
    assert_eq!(fifo[5], 3);
}
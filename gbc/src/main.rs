
extern "C"
{
    fn InitLib();
    fn DrawCycle()->i32;
}

fn main()
{
    unsafe
    {
        InitLib();
        //let colors:[u32;50*50] = [0x50505050;50*50];
        loop
        {
            if DrawCycle() == 0
            {
                break;
            }
        }
        
    }
}
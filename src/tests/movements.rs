use std::f32::consts::PI;
use std::thread::sleep;

#[test]
fn all_axes() {
    let mut syarm = crate::SyArm::from_conf(
        crate::JsonConfig::read_from_file("res/SyArm_Mk1.conf.json")
    );
    let dur = std::time::Duration::from_secs_f32(0.5);
    let angle = PI / 8.0;

    // syarm.update_sim();

    // syarm.debug_pins();

    println!("Running movement tests ... ");

    syarm.measure(2).unwrap();
    
    for i in 0 .. 4 {
        print!(" -> Moving Component {i} ... ");
        syarm.drive_comp_rel(i, angle);
        sleep(dur);
        syarm.drive_comp_rel(i, -angle);
        println!("Done!");
        sleep(dur);
    }
}
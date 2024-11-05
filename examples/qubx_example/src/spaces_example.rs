use qubx::qspaces::{ SpaceObject, SpaceMode, PolarPoint, QSpace };

pub fn spaces_example() {
    // let loc = vec![-30.0, 30.0, 90.0, 150.0, 210.0, 270.0];
    let loc = vec![-30.0, 30.0];
    let mut s = SpaceObject::new(&loc, SpaceMode::StereoCostantPower).unwrap();
    // let lpairs = s.pairs;
    // println!("Bases: {:?}", &lpairs);

    let mut spacer = QSpace::new(&mut s);

    let mut source = PolarPoint::new();
    source.set_theta(210.0);
    // source.set_r(0.7);
    // let vbap_gains = spacer.vbap(&source).unwrap();
    // println!("VBAP Gains: {:?}", vbap_gains);
    
    // let dbap_gains = spacer.dbap(&source, 6.0, None, None).unwrap();
    // println!("DBAP Gains: {:?}", dbap_gains);
    // println!("GEO CENTER: {:?}", s.geo_center);
    
    let source_angle = 0.0;
    let stereo_gains = spacer.stereo_pan(&source_angle).unwrap();
    println!("STEREO Gains: {:?}", stereo_gains);



}
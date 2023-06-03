use kelocam::object::contour::{Contour, Segment};

use kelocam::p2;

#[test]
pub fn line_bounding_box() {
    let a = Segment::Line {
        start: p2!(0, 0),
        end: p2!(1, 1),
    };
    let b = Segment::Line {
        start: p2!(0, 1),
        end: p2!(1, 0),
    };
    let c = Segment::Line {
        start: p2!(1, 0),
        end: p2!(0, 1),
    };
    let d = Segment::Line {
        start: p2!(1, 1),
        end: p2!(0, 0),
    };

    assert_eq!(a.bounding_box(), b.bounding_box());
    assert_eq!(b.bounding_box(), c.bounding_box());
    assert_eq!(c.bounding_box(), d.bounding_box());
}

#[test]
pub fn arc_bounding_box() {
    const HALF_PI: f64 = std::f64::consts::PI / 2.0;

    let a = Segment::Arc {
        center: p2!(0, 0),
        start: 0.0,
        end: HALF_PI,
        rad: 1.0,
    };
    let b = Segment::Arc {
        center: p2!(0, 0),
        start: HALF_PI,
        end: 0.0,
        rad: 1.0,
    };
    let c = Segment::Arc {
        center: p2!(0, 0),
        start: -HALF_PI,
        end: HALF_PI,
        rad: 1.0,
    };
    let d = Segment::Arc {
        center: p2!(0, 0),
        start: -3.0 * HALF_PI,
        end: 0.0,
        rad: 1.0,
    };

    assert_eq!(a.bounding_box(), (p2!(0, 0), p2!(1, 1)));
    assert_eq!(b.bounding_box(), (p2!(0, 0), p2!(1, 1)));
    assert_eq!(c.bounding_box(), (p2!(-1, 0), p2!(1, 1)));
    assert_eq!(d.bounding_box(), (p2!(-1, -1), p2!(1, 1)));
}

#[test]
pub fn contour_bounding_box() {
    const HALF_PI: f64 = std::f64::consts::PI / 2.0;

    let contour = Contour {
        segments: vec![
            Segment::Line {
                start: p2!(0, 0),
                end: p2!(1, 0),
            },
            Segment::Line {
                start: p2!(1, 0),
                end: p2!(1, 1),
            },
            Segment::Arc {
                start: 0.0,
                end: -HALF_PI,
                rad: 1.0,
                center: p2!(1, 0),
            },
        ],
        index: 0,
    };

    assert_eq!(contour.bounding_box(), (p2!(0, 0), p2!(1, 1)));
}

#[test]
pub fn contour_find_square() {
    let segments = vec![
        // Square 1
        Segment::Line {
            start: p2!(-1, -1),
            end: p2!(-1, 1),
        },
        Segment::Line {
            start: p2!(-1, 1),
            end: p2!(1, 1),
        },
        Segment::Line {
            start: p2!(1, 1),
            end: p2!(1, -1),
        },
        Segment::Line {
            start: p2!(1, -1),
            end: p2!(-1, -1),
        },
        // Square 2
        Segment::Line {
            start: p2!(-2, -2),
            end: p2!(-2, 2),
        },
        Segment::Line {
            start: p2!(-2, 2),
            end: p2!(2, 2),
        },
        Segment::Line {
            start: p2!(2, 2),
            end: p2!(2, -2),
        },
        Segment::Line {
            start: p2!(2, -2),
            end: p2!(-2, -2),
        },
    ];

    let contours = Contour::find(segments);

    println!("{:#?}", contours);

    assert_eq!(contours.len(), 2);
    assert_eq!(contours[0].segments.len(), 4);
}

#[test]
pub fn contour_find_circle() {
    const HALF_PI: f64 = std::f64::consts::PI / 2.0;

    let segments = vec![
        // Circle 1
        Segment::Arc {
            center: p2!(0, 0),
            start: 0.0,
            end: 4.0 * HALF_PI,
            rad: 1.0,
        },
        // Circle 2
        Segment::Arc {
            center: p2!(0, 0),
            start: 0.0,
            end: 4.0 * HALF_PI,
            rad: 2.0,
        },
    ];

    let contours = Contour::find(segments);

    println!("{:#?}", contours);

    assert_eq!(contours.len(), 2);
    assert_eq!(contours[0].segments.len(), 1);
}

#[test]
pub fn contour_index_square() {
    let segments = vec![
        // Square 1
        Segment::Line {
            start: p2!(-1, -1),
            end: p2!(-1, 1),
        },
        Segment::Line {
            start: p2!(-1, 1),
            end: p2!(1, 1),
        },
        Segment::Line {
            start: p2!(1, 1),
            end: p2!(1, -1),
        },
        Segment::Line {
            start: p2!(1, -1),
            end: p2!(-1, -1),
        },
        // Square 2
        Segment::Line {
            start: p2!(-2, -2),
            end: p2!(-2, 2),
        },
        Segment::Line {
            start: p2!(-2, 2),
            end: p2!(2, 2),
        },
        Segment::Line {
            start: p2!(2, 2),
            end: p2!(2, -2),
        },
        Segment::Line {
            start: p2!(2, -2),
            end: p2!(-2, -2),
        },
    ];

    let contours = Contour::index(Contour::find(segments));

    println!("{:#?}", contours);

    assert_eq!(contours.len(), 2);
    assert_eq!(contours[0].segments.len(), 4);

    // TODO: properly test
}

#[test]
pub fn contour_index_circle() {
    const HALF_PI: f64 = std::f64::consts::PI / 2.0;

    let segments = vec![
        // Circle 1
        Segment::Arc {
            center: p2!(0, 0),
            start: 0.0,
            end: 4.0 * HALF_PI,
            rad: 1.0,
        },
        // Circle 2
        Segment::Arc {
            center: p2!(0, 0),
            start: 0.0,
            end: 4.0 * HALF_PI,
            rad: 2.0,
        },
    ];

    let contours = Contour::index(Contour::find(segments));

    println!("{:#?}", contours);

    assert_eq!(contours.len(), 2);
    assert_eq!(contours[0].segments.len(), 1);

    // TODO: properly test
}

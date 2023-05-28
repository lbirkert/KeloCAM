use kelocam::object::point::Point2D;

#[test]
fn add_point() {
    let a = Point2D::from(5.0, -5.0);
    let b = Point2D::from(3.0, 2.0);

    assert_eq!(a + b, Point2D::from(8.0, -3.0));
    assert_eq!(b + a, Point2D::from(8.0, -3.0));
}

#[test]
fn sub_point() {
    let a = Point2D::from(5.0, -5.0);
    let b = Point2D::from(3.0, 2.0);

    assert_eq!(a - b, Point2D::from(2.0, -7.0));
    assert_eq!(b - a, Point2D::from(-2.0, 7.0));
}

#[test]
fn scale_point() {
    let a = Point2D::from(5.0, -5.0);

    assert_eq!(a * 5.0, Point2D::from(25.0, -25.0));
    assert_eq!(a * 3.0, Point2D::from(15.0, -15.0));
}

#[test]
fn mul_point() {
    let a = Point2D::from(5.0, -5.0);
    let b = Point2D::from(2.0, 4.0);

    assert_eq!(a * a, Point2D::from(25.0, 25.0));
    assert_eq!(a * b, Point2D::from(10.0, -20.0));
}

#[test]
fn min_point() {
    let a = Point2D::from(5.0, -5.0);
    let b = Point2D::from(2.0, 4.0);

    assert_eq!(a.min(b), Point2D::from(2.0, -5.0));
    assert_eq!(b.min(a), Point2D::from(2.0, -5.0));
}

#[test]
fn max_point() {
    let a = Point2D::from(5.0, -5.0);
    let b = Point2D::from(2.0, 4.0);

    assert_eq!(a.max(b), Point2D::from(5.0, 4.0));
    assert_eq!(b.max(a), Point2D::from(5.0, 4.0));
}

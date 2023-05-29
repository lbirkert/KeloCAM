use kelocam::p2;

#[test]
fn add_point() {
    let a = p2!(5, -5);
    let b = p2!(3, 2);

    assert_eq!(a + b, p2!(8, -3));
    assert_eq!(b + a, p2!(8, -3));
}

#[test]
fn sub_point() {
    let a = p2!(5, -5);
    let b = p2!(3, 2);

    assert_eq!(a - b, p2!(2, -7));
    assert_eq!(b - a, p2!(-2, 7));
}

#[test]
fn scale_point() {
    let a = p2!(5, -5);

    assert_eq!(a * 5.0, p2!(25, -25));
    assert_eq!(a * 3.0, p2!(15, -15));
}

#[test]
fn mul_point() {
    let a = p2!(5, -5);
    let b = p2!(2, 4);

    assert_eq!(a * a, p2!(25, 25));
    assert_eq!(a * b, p2!(10, -20));
}

#[test]
fn min_point() {
    let a = p2!(5, -5);
    let b = p2!(2, 4);

    assert_eq!(a.min(b), p2!(2, -5));
    assert_eq!(b.min(a), p2!(2, -5));
}

#[test]
fn max_point() {
    let a = p2!(5, -5);
    let b = p2!(2, 4);

    assert_eq!(a.max(b), p2!(5, 4));
    assert_eq!(b.max(a), p2!(5, 4));
}

#[test]
fn dist_point() {
    let a = p2!(5, -5);
    let b = p2!(9, -2);

    assert_eq!(a.dist(b), 5.0);
    assert_eq!(b.dist(a), 5.0);
}

#[test]
fn dist_approx_point() {
    let a = p2!(5, -5);
    let b = p2!(9, -2);

    assert_eq!(a.dist_approx(b), 7.0);
    assert_eq!(b.dist_approx(a), 7.0);
}

#[test]
fn greater_point() {
    let a = p2!(5, -2);
    let b = p2!(9, -5);
    let c = p2!(-6, -6);

    assert_eq!(a.greater(b), false);
    assert_eq!(b.greater(a), false);
    assert_eq!(a.greater(c), true);
    assert_eq!(b.greater(c), true);
}

#[test]
fn smaller_point() {
    let a = p2!(5, -2);
    let b = p2!(9, -5);
    let c = p2!(-6, -6);

    assert_eq!(a.smaller(b), false);
    assert_eq!(b.smaller(a), false);
    assert_eq!(c.smaller(a), true);
    assert_eq!(c.smaller(b), true);
}

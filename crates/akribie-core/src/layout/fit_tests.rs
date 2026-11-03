//! Caixa de texto segura: os quatro cantos da área do rótulo ficam dentro da forma.

#[test]
fn safe_box_corners_stay_inside_the_shape() {
    use super::tests::placed;
    use super::text_box;
    use crate::geometry::Rect;
    use crate::model::Shape;
    use crate::sketch::cylinder_radius;

    fn inside_convex(point: (f32, f32), polygon: &[(f32, f32)]) -> bool {
        let count = polygon.len();
        let (mut negative, mut positive) = (false, false);
        for index in 0..count {
            let (a, b) = (polygon[index], polygon[(index + 1) % count]);
            let cross = (b.0 - a.0) * (point.1 - a.1) - (b.1 - a.1) * (point.0 - a.0);
            if cross < -0.01 {
                negative = true;
            }
            if cross > 0.01 {
                positive = true;
            }
        }
        !(negative && positive)
    }

    fn inside(shape: Shape, rect: Rect, point: (f32, f32)) -> bool {
        let (cx, cy) = rect.center();
        let (rx, ry) = (rect.w * 0.5, rect.h * 0.5);
        let (dx, dy) = ((point.0 - cx) / rx, (point.1 - cy) / ry);
        match shape {
            Shape::Circle => dx * dx + dy * dy <= 1.001,
            Shape::Cloud => (dx / 0.9).powi(2) + (dy / 0.9).powi(2) <= 1.001,
            Shape::Diamond => dx.abs() + dy.abs() <= 1.001,
            Shape::Hexagon => {
                let quarter = rect.w * 0.25;
                let polygon = [
                    (rect.x + quarter, rect.y),
                    (rect.right() - quarter, rect.y),
                    (rect.right(), cy),
                    (rect.right() - quarter, rect.bottom()),
                    (rect.x + quarter, rect.bottom()),
                    (rect.x, cy),
                ];
                inside_convex(point, &polygon)
            }
            Shape::Cylinder => {
                let ry = cylinder_radius(rect);
                point.0 >= rect.x - 0.01
                    && point.0 <= rect.right() + 0.01
                    && point.1 >= rect.y + 2.0 * ry - 0.01
                    && point.1 <= rect.bottom() - ry + 0.01
            }
            Shape::Rect | Shape::Rounded | Shape::Note | Shape::Text => {
                point.0 >= rect.x - 0.01
                    && point.0 <= rect.right() + 0.01
                    && point.1 >= rect.y - 0.01
                    && point.1 <= rect.bottom() + 0.01
            }
        }
    }

    for shape in [
        Shape::Rect,
        Shape::Rounded,
        Shape::Circle,
        Shape::Diamond,
        Shape::Cloud,
        Shape::Cylinder,
        Shape::Hexagon,
        Shape::Note,
    ] {
        let placed = placed(&format!(
            "chart v1 \"x\" layout=grid\nnode a \"Rótulo comprido\" shape={}\n",
            shape.name()
        ));
        let node = placed.nodes.first().expect("nó");
        let box_ = text_box(shape, node.rect);
        let corners = [
            (box_.x, box_.y),
            (box_.right(), box_.y),
            (box_.right(), box_.bottom()),
            (box_.x, box_.bottom()),
        ];
        for corner in corners {
            assert!(
                inside(shape, node.rect, corner),
                "{}: canto {corner:?} fora da forma (caixa {box_:?})",
                shape.name()
            );
        }
    }
}

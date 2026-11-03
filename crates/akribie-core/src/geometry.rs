//! Geometria pura: retângulos, interseção de contorno e rotas de aresta.
//!
//! Nada aqui conhece documento, cor ou SVG: só números. É o tipo de módulo que se testa por
//! invariante (monotonicidade, simetria, contenção), e é o que mantém o layout verificável.

use crate::model::Shape;

/// Ponto no plano do canvas (origem no canto superior esquerdo, y para baixo).
pub type Point = (f32, f32);

/// Retângulo alinhado aos eixos.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Canto esquerdo.
    pub x: f32,
    /// Canto superior.
    pub y: f32,
    /// Largura.
    pub w: f32,
    /// Altura.
    pub h: f32,
}

impl Rect {
    /// Constrói um retângulo.
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// Centro do retângulo.
    pub fn center(&self) -> Point {
        (self.x + self.w * 0.5, self.y + self.h * 0.5)
    }

    /// Coordenada da borda direita.
    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    /// Coordenada da borda inferior.
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    /// Menor retângulo que contém os dois.
    pub fn union(&self, other: Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Self::new(x, y, right - x, bottom - y)
    }

    /// Cresce (ou encolhe, com valor negativo) em todas as bordas.
    pub fn inflate(&self, by: f32) -> Self {
        Self::new(
            self.x - by,
            self.y - by,
            self.w + 2.0 * by,
            self.h + 2.0 * by,
        )
    }

    /// Translada sem alterar o tamanho.
    pub fn translate(&self, dx: f32, dy: f32) -> Self {
        Self::new(self.x + dx, self.y + dy, self.w, self.h)
    }

    /// `true` quando o ponto está dentro (bordas incluídas).
    pub fn contains(&self, point: Point) -> bool {
        point.0 >= self.x
            && point.0 <= self.right()
            && point.1 >= self.y
            && point.1 <= self.bottom()
    }
}

/// Ponto onde o raio de `center` até `toward` cruza o contorno da forma.
///
/// `rect` é a caixa da forma; `circle` usa o círculo inscrito, `diamond` a losango inscrito e as
/// demais formas o retângulo (aproximação declarada — nenhuma aresta "entra" na forma).
pub fn outline_hit(rect: Rect, shape: Shape, toward: Point) -> Point {
    let (cx, cy) = rect.center();
    let dx = toward.0 - cx;
    let dy = toward.1 - cy;
    if dx.abs() < f32::EPSILON && dy.abs() < f32::EPSILON {
        return (cx, cy);
    }
    let half_w = (rect.w * 0.5).max(0.5);
    let half_h = (rect.h * 0.5).max(0.5);
    let scale = match shape {
        Shape::Circle => half_w.min(half_h) / (dx * dx + dy * dy).sqrt(),
        Shape::Diamond => 1.0 / (dx.abs() / half_w + dy.abs() / half_h),
        _ => 1.0 / (dx.abs() / half_w).max(dy.abs() / half_h),
    };
    (cx + dx * scale, cy + dy * scale)
}

/// Ponto médio da rota por comprimento de arco, com a direção local (unitária).
///
/// É o que ancora rótulos de aresta: em rota de dois pontos dá o meio exato; numa curva amostrada
/// dá o ponto a meio caminho — nunca o fim da rota, como acontecia ao indexar `len / 2`.
pub fn route_midpoint(route: &[Point]) -> Option<(Point, Point)> {
    let mut total = 0.0_f32;
    for pair in route.windows(2) {
        let (from, to) = (pair.first()?, pair.get(1)?);
        total += (to.0 - from.0).hypot(to.1 - from.1);
    }
    let first = route.first().copied()?;
    if total <= f32::EPSILON {
        return Some((first, (1.0, 0.0)));
    }
    let half = total * 0.5;
    let mut walked = 0.0_f32;
    for pair in route.windows(2) {
        let (from, to) = (pair.first()?, pair.get(1)?);
        let length = (to.0 - from.0).hypot(to.1 - from.1);
        if length <= f32::EPSILON {
            continue;
        }
        if walked + length >= half {
            let t = (half - walked) / length;
            let point = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
            return Some((point, ((to.0 - from.0) / length, (to.1 - from.1) / length)));
        }
        walked += length;
    }
    route.last().copied().map(|last| (last, (1.0, 0.0)))
}

/// Rota em linha reta.
pub fn route_line(from: Point, to: Point) -> Vec<Point> {
    vec![from, to]
}

/// Rota em curva quadrática: controle a 15% da distância, na perpendicular do meio.
pub fn route_arc(from: Point, to: Point) -> Vec<Point> {
    const SAMPLES: u8 = 16;
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let control = (from.0 + dx * 0.5 - dy * 0.15, from.1 + dy * 0.5 + dx * 0.15);
    let last = f32::from(SAMPLES);
    (0..=SAMPLES)
        .map(|index| quadratic(from, control, to, f32::from(index) / last))
        .collect()
}

fn quadratic(from: Point, control: Point, to: Point, t: f32) -> Point {
    let inverse = 1.0 - t;
    let a = inverse * inverse;
    let b = 2.0 * inverse * t;
    let c = t * t;
    (
        a * from.0 + b * control.0 + c * to.0,
        a * from.1 + b * control.1 + c * to.1,
    )
}

/// Rota em cotovelo: `H-V-H` quando o deslocamento horizontal domina, senão `V-H-V`.
pub fn route_elbow(from: Point, to: Point) -> Vec<Point> {
    let dx = (to.0 - from.0).abs();
    let dy = (to.1 - from.1).abs();
    if dx >= dy {
        let mid = f32::midpoint(from.0, to.0);
        vec![from, (mid, from.1), (mid, to.1), to]
    } else {
        let mid = f32::midpoint(from.1, to.1);
        vec![from, (from.0, mid), (to.0, mid), to]
    }
}

/// Laço de auto-aresta: semicírculo de raio 32 no lado direito do nó.
pub fn self_edge(rect: Rect) -> Vec<Point> {
    const RADIUS: f32 = 32.0;
    const SAMPLES: u8 = 12;
    let (_, cy) = rect.center();
    let anchor_x = rect.right();
    let last = f32::from(SAMPLES);
    (0..=SAMPLES)
        .map(|index| {
            let t = f32::from(index) / last;
            let angle = -0.5 * std::f32::consts::PI + std::f32::consts::PI * t;
            (anchor_x + RADIUS * angle.cos(), cy + RADIUS * angle.sin())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_and_inflate_cover_both_rects() {
        let a = Rect::new(10.0, 10.0, 20.0, 20.0);
        let b = Rect::new(50.0, 40.0, 10.0, 10.0);
        let union = a.union(b);
        assert!(union.contains(a.center()));
        assert!(union.contains(b.center()));
        assert_eq!(union.right(), 60.0);
        assert_eq!(union.bottom(), 50.0);
        let grown = a.inflate(5.0);
        assert_eq!(grown, Rect::new(5.0, 5.0, 30.0, 30.0));
    }

    #[test]
    fn outline_hits_each_edge_of_the_rect() {
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
        let center = rect.center();
        let right = outline_hit(rect, Shape::Rect, (500.0, center.1));
        assert!((right.0 - 100.0).abs() < 0.01, "{right:?}");
        let top = outline_hit(rect, Shape::Rect, (center.0, -500.0));
        assert!(top.1.abs() < 0.01, "{top:?}");
    }

    #[test]
    fn circle_hit_is_on_the_inscribed_circle() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let hit = outline_hit(rect, Shape::Circle, (1000.0, 50.0));
        assert!((hit.0 - 100.0).abs() < 0.01, "{hit:?}");
        let diagonal = outline_hit(rect, Shape::Circle, (150.0, 150.0));
        let distance = ((diagonal.0 - 50.0).powi(2) + (diagonal.1 - 50.0).powi(2)).sqrt();
        assert!((distance - 50.0).abs() < 0.01, "{distance}");
    }

    #[test]
    fn diamond_hit_is_inside_the_corner() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let hit = outline_hit(rect, Shape::Diamond, (150.0, 150.0));
        assert!(hit.0 < 100.0 && hit.1 < 100.0, "{hit:?}");
    }

    #[test]
    fn elbow_turns_at_the_middle() {
        let route = route_elbow((0.0, 0.0), (100.0, 10.0));
        assert_eq!(route.len(), 4);
        assert_eq!(route[1], (50.0, 0.0));
        let route = route_elbow((0.0, 0.0), (10.0, 100.0));
        assert_eq!(route[1], (0.0, 50.0));
    }

    #[test]
    fn arc_starts_and_ends_at_the_endpoints() {
        let route = route_arc((0.0, 0.0), (100.0, 0.0));
        assert_eq!(route.first().copied(), Some((0.0, 0.0)));
        assert_eq!(route.last().copied(), Some((100.0, 0.0)));
        assert!(route.len() > 2);
        assert!(
            route.iter().any(|point| point.1.abs() > 1.0),
            "curva deve sair da reta"
        );
    }

    #[test]
    fn self_edge_bows_to_the_right_of_the_node() {
        let rect = Rect::new(0.0, 0.0, 100.0, 40.0);
        let route = self_edge(rect);
        assert!(route.iter().all(|point| point.0 >= 99.0), "{route:?}");
        assert!(route.iter().any(|point| point.1 < 20.0));
        assert!(route.iter().any(|point| point.1 > 20.0));
    }
}

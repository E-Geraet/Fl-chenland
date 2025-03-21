use rand::Rng;
use nannou::prelude::*;
use std::collections::HashMap;

// Struktur für eine geometrische Form
struct Shape {
    sides: u32,
    position: Point2,
    size: f32,
    color: Rgb,
    velocity: Vec2,
    age: f32,
}

impl Shape {
    fn new(sides: u32, position: Point2) -> Self {
        let mut rng = rand::thread_rng();

        // Farbe basierend auf der Anzahl der Seiten
        let hue = map_range(sides as f32, 3.0, 8.0, 0.0, 1.0);
        let color = hsv(hue, 0.8, 0.9).into();

        Shape {
            sides,
            position,
            size: map_range(sides as f32, 3.0, 8.0, 15.0, 30.0), // Größe nach Status
            color,
            velocity: vec2(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)),
            age: 0.0,
        }
    }

    // Bewegung der Form
    fn update(&mut self, bounds: Rect) {
        self.position += self.velocity;
        self.age += 0.01;

        // Abprallen an den Grenzen
        if self.position.x < bounds.left() + self.size || self.position.x > bounds.right() - self.size {
            self.velocity.x *= -1.0;
        }
        if self.position.y < bounds.bottom() + self.size || self.position.y > bounds.top() - self.size {
            self.velocity.y *= -1.0;
        }
    }

    // Überprüfen, ob zwei Formen sich berühren
    fn collides_with(&self, other: &Shape) -> bool {
        let distance = self.position.distance(other.position);
        distance < self.size + other.size
    }

    // Zeichnen der Form
    fn display(&self, draw: &Draw) {
        let points = (0..self.sides).map(|i| {
            let angle = 2.0 * PI * i as f32 / self.sides as f32;
            let x = self.size * angle.cos();
            let y = self.size * angle.sin();
            pt2(x, y)
        }).collect::<Vec<_>>();

        draw.polygon()
            .color(self.color)
            .points(points)
            .xy(self.position);
    }
}

struct Model {
    shapes: Vec<Shape>,
    reproduction_cooldown: f32,
    shape_counts: HashMap<u32, usize>,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(800, 600)
        .title("Flächenland Simulation")
        .view(view)
        .build()
        .unwrap();

    let mut shapes = Vec::new();
    let mut shape_counts = HashMap::new();

    let mut rng = rand::thread_rng();

    // Erzeugen von initialen Formen
    for _ in 0..20 {
        let sides = rng.gen_range(3..=6);
        let x = rng.gen_range(-300.0..300.0);
        let y = rng.gen_range(-200.0..200.0);

        *shape_counts.entry(sides).or_insert(0) += 1;
        shapes.push(Shape::new(sides, pt2(x, y)));
    }

    Model {
        shapes,
        reproduction_cooldown: 0.0,
        shape_counts,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let bounds = app.window_rect();
    let mut rng = rand::thread_rng();

    // Aktualisieren aller Formen
    for shape in &mut model.shapes {
        shape.update(bounds);
    }

    model.reproduction_cooldown -= 0.01;

    // Überprüfen auf Kollisionen und mögliche Fortpflanzung
    if model.reproduction_cooldown <= 0.0 {
        let mut new_shapes = Vec::new();

        for i in 0..model.shapes.len() {
            for j in i+1..model.shapes.len() {
                // Prüfen, ob zwei Vierecke kollidieren
                if model.shapes[i].sides == 4 && model.shapes[j].sides == 4 &&
                    model.shapes[i].collides_with(&model.shapes[j]) &&
                    model.shapes[i].age > 5.0 && model.shapes[j].age > 5.0 {

                    // 50% Chance für Dreieck oder Fünfeck als Kind
                    let child_sides = if rng.gen_bool(0.5) { 3 } else { 5 };

                    // Position des Kindes zwischen den Eltern
                    let child_pos = pt2(
                        (model.shapes[i].position.x + model.shapes[j].position.x) / 2.0,
                        (model.shapes[i].position.y + model.shapes[j].position.y) / 2.0
                    );

                    let new_shape = Shape::new(child_sides, child_pos);
                    new_shapes.push(new_shape);

                    *model.shape_counts.entry(child_sides).or_insert(0) += 1;
                    model.reproduction_cooldown = 3.0; // Cooldown nach Fortpflanzung
                    break;
                }
            }
        }

        // Hinzufügen neuer Formen
        model.shapes.extend(new_shapes);
    }

    // Begrenzen der Bevölkerung
    while model.shapes.len() > 100 {
        // Entfernen der ältesten Form
        if let Some(oldest_idx) = model.shapes.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.age.partial_cmp(&b.age).unwrap())
            .map(|(i, _)| i) {
            let sides = model.shapes[oldest_idx].sides;
            if let Some(count) = model.shape_counts.get_mut(&sides) {
                *count -= 1;
            }
            model.shapes.remove(oldest_idx);
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    // Zeichnen aller Formen
    for shape in &model.shapes {
        shape.display(&draw);
    }

    // Anzeigen der Statistik
    let mut y_pos = app.window_rect().top() - 20.0;
    for sides in 3..=7 {
        let count = model.shape_counts.get(&sides).unwrap_or(&0);
        let text = format!("{}-Ecke: {}", sides, count);
        draw.text(&text)
            .x_y(-app.window_rect().right() + 80.0, y_pos)
            .color(WHITE);
        y_pos -= 20.0;
    }

    draw.to_frame(app, &frame).unwrap();
}

fn main() {
    nannou::app(model)
        .update(update)
        .run();
}
use rand::Rng;
use nannou::prelude::*;
use std::collections::HashMap;

// Enum for different agent types
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SocialClass {
    SoldierOrWorkman, // For Isosceles (currently not differentiated, will be 3-sided non-regular)
    Craftsman,        // Equilateral Triangles (currently all 3-sided)
    Gentleman,        // Squares, Pentagons
    NobilityLow,      // Hexagons
    NobilityMid,      // 7-9 sides
    NobilityHigh,     // 10-11 sides
    Priest,           // 12+ sides
}

// Helper function to get all social classes for display
fn all_social_classes() -> Vec<SocialClass> {
    vec![
        SocialClass::SoldierOrWorkman, // Will be 0 until Isosceles are differentiated
        SocialClass::Craftsman,
        SocialClass::Gentleman,
        SocialClass::NobilityLow,
        SocialClass::NobilityMid,
        SocialClass::NobilityHigh,
        SocialClass::Priest,
    ]
}


#[derive(Clone, Debug, PartialEq)]
pub enum AgentType {
    Polygon { sides: u32, is_regular: bool, social_class: SocialClass, smallest_angle_degrees: Option<f32> },
    Line { length: f32 },
}

fn get_social_class(sides: u32, is_regular: bool, smallest_angle_degrees: Option<f32>) -> SocialClass {
    if smallest_angle_degrees.is_some() && sides == 3 { // Isosceles Triangle (already implies !is_regular from constructor)
        return SocialClass::SoldierOrWorkman;
    } else if !is_regular { // General Irregular Polygon (not Isosceles)
        return SocialClass::SoldierOrWorkman;
    }
    // Regular Polygons (including Equilateral Triangles)
    match sides {
        3 => SocialClass::Craftsman, // Equilateral Triangle
        4 | 5 => SocialClass::Gentleman,
        6 => SocialClass::NobilityLow,
        7..=9 => SocialClass::NobilityMid,
        10..=11 => SocialClass::NobilityHigh,
        12.. => SocialClass::Priest,
        _ => SocialClass::SoldierOrWorkman, // Default for <3 sides, though children are ensured to be >=3
    }
}

// Struktur für eine geometrische Form
struct Shape {
    agent_type: AgentType,
    position: Point2,
    size: f32, // Bounding radius
    color: Rgb,
    velocity: Vec2,
    age: f32,
    is_moving: bool, // For Line's peace-cry
}

impl Shape {
    fn new(mut agent_type: AgentType, position: Point2) -> Self {
        let mut rng = rand::thread_rng();
        let color;
        let size;

        match &mut agent_type {
            AgentType::Polygon { sides, is_regular, social_class, smallest_angle_degrees } => {
                // Determine regularity based on angle for triangles
                if *sides == 3 && smallest_angle_degrees.is_some() {
                    *is_regular = false;
                } else if *sides == 3 && smallest_angle_degrees.is_none() {
                    *is_regular = true; // Equilateral
                }
                // For polygons with sides > 3, is_regular is assumed to be set correctly upon creation.

                *social_class = get_social_class(*sides, *is_regular, *smallest_angle_degrees);
                
                // Greyscale color based on SocialClass to represent hierarchy, adhering to "no color" lore.
                color = get_class_color(*social_class);
                size = map_range(*sides as f32, 3.0, 12.0, 15.0, 40.0); // Adjusted size range
            }
            AgentType::Line { length } => {
                // Lines (Women) are a very light grey, distinct from Priests.
                color = Rgb::new(0.95, 0.95, 0.95); 
                size = length / 2.0;
            }
        }

        Shape {
            agent_type,
            position,
            size,
            color,
            velocity: vec2(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)),
            age: 0.0,
            is_moving: false,
        }
    }

    // Bewegung der Form
    fn update(&mut self, bounds: Rect) {
        self.position += self.velocity;
        
        // Aging process based on regularity
        if let AgentType::Polygon { is_regular: false, smallest_angle_degrees: None, .. } = self.agent_type {
            self.age += 0.05; // Age faster if irregular (and not Isosceles)
        } else {
            self.age += 0.01; // Normal aging
        }

        // Update is_moving status based on velocity
        self.is_moving = self.velocity.length_squared() > 0.01; // Threshold of 0.1^2

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
        match self.agent_type {
            AgentType::Polygon { sides, is_regular, smallest_angle_degrees, .. } => {
                let points = if sides == 3 && !is_regular && smallest_angle_degrees.is_some() {
                    // Isosceles Triangle Drawing
                    let alpha_rad = smallest_angle_degrees.unwrap().to_radians(); // Smallest angle
                    let beta_rad = (PI - 2.0 * alpha_rad) / 1.0; // The third angle (apex if alpha is base)
                                                              // This logic assumes alpha is one of the two equal angles.
                                                              // If alpha is the apex, then beta = (PI - alpha_rad)/2.0 are the base angles.
                                                              // For simplicity, let's assume smallest_angle_degrees refers to one of the two equal base angles.
                                                              // So, the angles are alpha, alpha, (180 - 2*alpha).
                    
                    // Simplified drawing: Use self.size as length of the two equal sides.
                    // Place base center at (0,0) for calculation before rotating/translating.
                    // Apex will be at (0, y_apex). Base vertices at (-x_base, y_base) and (x_base, y_base).
                    // This is a common way to draw an isosceles triangle.
                    // For this example, let's make the base horizontal.
                    // The height 'h' and half-base 'b_half' can be found using trigonometry.
                    // h = self.size * alpha_rad.sin();
                    // b_half = self.size * alpha_rad.cos();
                    // Apex: (0, self.size * (PI - beta_rad / 2.0).sin()) // This is getting complicated quickly.

                    // Simpler approach for now:
                    // Define points for a triangle that is clearly not equilateral.
                    // One point at top, two at bottom. self.size can be height.
                    // This is a placeholder for more accurate geometric drawing.
                    vec![
                        pt2(0.0, self.size * 0.7), // Apex
                        pt2(-self.size * 0.5, -self.size * 0.3), // Bottom-left
                        pt2(self.size * 0.5, -self.size * 0.3)  // Bottom-right
                    ]
                } else if !is_regular && smallest_angle_degrees.is_none() {
                    // General Irregular Polygon Drawing (not Isosceles)
                    let mut rng_display = rand::thread_rng(); // For irregular vertex perturbation
                    (0..sides).map(|i| {
                        let angle = 2.0 * PI * i as f32 / sides as f32;
                        let base_x = self.size * angle.cos();
                        let base_y = self.size * angle.sin();
                        // Add random perturbation
                        let offset_x = self.size * 0.15 * rng_display.gen_range(-1.0..1.0); // Increased perturbation
                        let offset_y = self.size * 0.15 * rng_display.gen_range(-1.0..1.0);
                        pt2(base_x + offset_x, base_y + offset_y)
                    }).collect::<Vec<_>>()
                }
                 else {
                    // Regular Polygon Drawing (including Equilateral Triangles)
                    (0..sides).map(|i| {
                        let angle = 2.0 * PI * i as f32 / sides as f32;
                        let x = self.size * angle.cos();
                        let y = self.size * angle.sin();
                        pt2(x, y)
                    }).collect::<Vec<_>>()
                };

                draw.polygon()
                    .color(self.color)
                    .points(points)
                    .xy(self.position);
            }
            AgentType::Line { length } => {
                // Draw a simple horizontal line
                draw.line()
                    .start(pt2(-length / 2.0, 0.0))
                    .end(pt2(length / 2.0, 0.0))
                    .weight(2.0)
                    .color(self.color)
                    .xy(self.position);

                // Peace-cry visual for moving lines
                if self.is_moving {
                    draw.ellipse()
                        .xy(self.position)
                        .radius(self.size * 0.5) // Smaller radius for the cry indicator
                        .color(srgba(0.8, 0.8, 0.8, 0.3)); // Semi-transparent white
                }
            }
        }
    }
}

struct Model {
    shapes: Vec<Shape>,
    reproduction_cooldown: f32,
    shape_counts: HashMap<SocialClass, usize>,
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

    // Erzeugen von initialen Polygonen (Regular ones)
    for _ in 0..15 { // Reduced count to make space for Isosceles
        let sides = rng.gen_range(3..=6); 
        let x = rng.gen_range(-300.0..300.0);
        let y = rng.gen_range(-200.0..200.0);
        let is_regular = true; // These are regular polygons
        let smallest_angle_degrees = None; // Not Isosceles
        let social_class = get_social_class(sides, is_regular, smallest_angle_degrees);

        let agent_type = AgentType::Polygon { sides, is_regular, social_class, smallest_angle_degrees };
        *shape_counts.entry(social_class).or_insert(0) += 1;
        shapes.push(Shape::new(agent_type, pt2(x, y)));
    }
    
    // Erzeugen von initialen Isosceles Triangles (Soldiers/Workmen)
    for _ in 0..5 { // Spawn 5 Isosceles triangles
        let x = rng.gen_range(-300.0..300.0);
        let y = rng.gen_range(-200.0..200.0);
        let sides = 3;
        let is_regular = false; // Isosceles are not regular
        let smallest_angle_degrees = Some(45.0); // Example starting angle
        let social_class = get_social_class(sides, is_regular, smallest_angle_degrees); // Should be SoldierOrWorkman

        let agent_type = AgentType::Polygon { sides, is_regular, social_class, smallest_angle_degrees };
        *shape_counts.entry(social_class).or_insert(0) += 1;
        shapes.push(Shape::new(agent_type, pt2(x,y)));
    }


    // Erzeugen von initialen Linien (Frauen)
    for _ in 0..10 { // Initial Line count
        let x = rng.gen_range(-300.0..300.0);
        let y = rng.gen_range(-200.0..200.0);
        let agent_type = AgentType::Line { length: 30.0 };
        // Lines are not tracked in shape_counts for now
        shapes.push(Shape::new(agent_type, pt2(x,y)));
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
            for j in i + 1..model.shapes.len() {
                let shape_i_agent_type = model.shapes[i].agent_type.clone(); // Clone to avoid borrow issues
                let shape_j_agent_type = model.shapes[j].agent_type.clone(); // Clone to avoid borrow issues

                if let (
                    AgentType::Polygon { sides: sides1, is_regular: is_regular1, social_class: _, smallest_angle_degrees: angle1 },
                    AgentType::Polygon { sides: _sides2, is_regular: _is_regular2, social_class: _, smallest_angle_degrees: _angle2 }
                ) = (shape_i_agent_type, shape_j_agent_type) { // Using cloned types
                    
                    if model.shapes[i].collides_with(&model.shapes[j]) &&
                       model.shapes[i].age > 5.0 && model.shapes[j].age > 5.0 {

                        let child_agent_type: AgentType;

                        // Determine father (shape i) and its properties
                        if sides1 == 3 && !is_regular1 && angle1.is_some() { // Father is Isosceles
                            let current_angle = angle1.unwrap();
                            let new_angle = current_angle + 0.5;

                            if new_angle >= 59.9 { // Transition to Equilateral
                                let mut new_is_regular = true;
                                if rng.gen_bool(0.1) { // 10% chance for new Equilateral to be born irregular
                                    new_is_regular = false;
                                }
                                child_agent_type = AgentType::Polygon {
                                    sides: 3,
                                    is_regular: new_is_regular, // Could be irregular Equilateral
                                    social_class: get_social_class(3, new_is_regular, None),
                                    smallest_angle_degrees: None, // No longer Isosceles
                                };
                            } else { // Stays Isosceles
                                child_agent_type = AgentType::Polygon { // Isosceles children are not subject to the new general irregularity chance
                                    sides: 3,
                                    is_regular: false, // Isosceles are by definition not regular in the general sense
                                    social_class: get_social_class(3, false, Some(new_angle)), // Soldier/Workman
                                    smallest_angle_degrees: Some(new_angle),
                                };
                            }
                        } else { // Father is Regular Polygon (or Equilateral Triangle)
                            let father_sides = sides1;
                            let mut child_sides = father_sides + 1;
                            if child_sides > 12 { child_sides = 12; }
                            if child_sides < 3 { child_sides = 3; }
                            
                            let mut child_is_regular = true; // Normally regular
                            if rng.gen_bool(0.1) { // 10% chance to become irregular
                                child_is_regular = false;
                            }
                            let child_smallest_angle_degrees = None; // Not Isosceles
                            let child_social_class = get_social_class(child_sides, child_is_regular, child_smallest_angle_degrees);

                            child_agent_type = AgentType::Polygon {
                                sides: child_sides,
                                is_regular: child_is_regular,
                                social_class: child_social_class,
                                smallest_angle_degrees: child_smallest_angle_degrees,
                            };
                        }

                        // Position des Kindes zwischen den Eltern
                        let child_pos = pt2(
                            (model.shapes[i].position.x + model.shapes[j].position.x) / 2.0,
                            (model.shapes[i].position.y + model.shapes[j].position.y) / 2.0
                        );
                        
                        let new_shape = Shape::new(child_agent_type.clone(), child_pos); // Pass cloned type
                        new_shapes.push(new_shape);

                        // Update counts based on the actual class of the child
                        if let AgentType::Polygon{social_class: actual_child_class, ..} = child_agent_type {
                             *model.shape_counts.entry(actual_child_class).or_insert(0) += 1;
                        }
                       
                        model.reproduction_cooldown = 3.0; // Cooldown nach Fortpflanzung
                        break; // Exit inner loop once reproduction occurs
                    }
                }
            }
            if model.reproduction_cooldown > 0.0 { // If a child was born, break outer loop too
                break;
            }
        }

        // Hinzufügen neuer Formen
        model.shapes.extend(new_shapes);
    }

// Helper function to determine greyscale color based on SocialClass
fn get_class_color(social_class: SocialClass) -> Rgb {
    // Greyscale color scheme: brightness indicates social hierarchy.
    // Adheres to the "no color" lore of Flatland.
    match social_class {
        SocialClass::Priest => Rgb::new(0.9, 0.9, 0.9),           // Very light grey
        SocialClass::NobilityHigh => Rgb::new(0.8, 0.8, 0.8),     // Light grey
        SocialClass::NobilityMid => Rgb::new(0.7, 0.7, 0.7),      // Medium-light grey
        SocialClass::NobilityLow => Rgb::new(0.6, 0.6, 0.6),      // Medium grey
        SocialClass::Gentleman => Rgb::new(0.5, 0.5, 0.5),        // Medium-dark grey
        SocialClass::Craftsman => Rgb::new(0.4, 0.4, 0.4),        // Dark grey
        SocialClass::SoldierOrWorkman => Rgb::new(0.3, 0.3, 0.3), // Very dark grey
    }
}

// Begrenzen der Bevölkerung
    while model.shapes.len() > 100 {
        // Entfernen der ältesten Form
        if let Some(oldest_idx) = model.shapes.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.age.partial_cmp(&b.age).unwrap())
            .map(|(i, _)| i) {
            
            match model.shapes[oldest_idx].agent_type {
                AgentType::Polygon { social_class, .. } => {
                    if let Some(count) = model.shape_counts.get_mut(&social_class) {
                        if *count > 0 { // Ensure count doesn't go below zero
                           *count -= 1;
                        }
                    }
                }
                AgentType::Line { .. } => {
                    // Lines are not currently tracked in shape_counts
                }
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
    
    for social_class in all_social_classes() {
        let count = model.shape_counts.get(&social_class).unwrap_or(&0);
        let class_name = format!("{:?}", social_class); // Get enum variant name as string
        let text = format!("{}: {}", class_name, count);
        draw.text(&text)
            .x_y(-app.window_rect().right() + 100.0, y_pos) // Adjusted x for longer names
            .font_size(12)
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
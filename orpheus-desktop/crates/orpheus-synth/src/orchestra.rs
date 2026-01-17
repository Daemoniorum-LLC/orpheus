//! Orchestral mixer with spatial positioning and concert hall simulation
//!
//! Provides:
//! - Concert hall acoustic modeling
//! - Musician positioning in 3D space
//! - Spatial audio processing (panning, distance, early reflections)
//! - Section management for full orchestra
//! - Visualizer data for UI rendering

use std::f32::consts::PI;

/// 3D position in the concert hall (meters from center stage)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    /// Left-right position (-x = stage left, +x = stage right)
    pub x: f32,
    /// Front-back position (-y = downstage/audience, +y = upstage/back)
    pub y: f32,
    /// Height (0 = stage floor, positive = elevated)
    pub z: f32,
}

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Center stage position
    pub fn center() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Distance from another position
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Angle from listener (radians, 0 = front, positive = right)
    pub fn angle_from(&self, listener: &Position) -> f32 {
        let dx = self.x - listener.x;
        let dy = self.y - listener.y;
        dx.atan2(dy)
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::center()
    }
}

/// Listener/audience position
#[derive(Debug, Clone, Copy)]
pub struct Listener {
    pub position: Position,
    /// Head rotation in radians (0 = facing stage)
    pub rotation: f32,
}

impl Default for Listener {
    fn default() -> Self {
        Self {
            // Typical conductor position, or center of audience
            position: Position::new(0.0, -8.0, 1.2),
            rotation: 0.0,
        }
    }
}

/// Concert hall types with different acoustic characteristics
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HallType {
    /// Intimate recital hall (300-600 seats)
    RecitalHall,
    /// Medium concert hall (800-1200 seats)
    ChamberHall,
    /// Large symphony hall (1500-2500 seats)
    SymphonyHall,
    /// Very large concert hall (2500+ seats)
    GrandHall,
    /// Cathedral/church acoustic
    Cathedral,
    /// Outdoor amphitheater
    Amphitheater,
    /// Recording studio (dry)
    Studio,
    /// Custom hall
    Custom,
}

impl HallType {
    /// Get hall dimensions (width, depth, height) in meters
    pub fn dimensions(&self) -> (f32, f32, f32) {
        match self {
            Self::RecitalHall => (15.0, 20.0, 8.0),
            Self::ChamberHall => (22.0, 30.0, 12.0),
            Self::SymphonyHall => (30.0, 45.0, 18.0),
            Self::GrandHall => (40.0, 60.0, 25.0),
            Self::Cathedral => (25.0, 80.0, 30.0),
            Self::Amphitheater => (50.0, 40.0, 0.0), // Open air
            Self::Studio => (12.0, 15.0, 4.0),
            Self::Custom => (25.0, 35.0, 15.0),
        }
    }

    /// Get reverb time (RT60) in seconds
    pub fn reverb_time(&self) -> f32 {
        match self {
            Self::RecitalHall => 1.4,
            Self::ChamberHall => 1.8,
            Self::SymphonyHall => 2.2,
            Self::GrandHall => 2.8,
            Self::Cathedral => 4.5,
            Self::Amphitheater => 0.3,
            Self::Studio => 0.4,
            Self::Custom => 2.0,
        }
    }

    /// Get early reflection density (0-1)
    pub fn reflection_density(&self) -> f32 {
        match self {
            Self::RecitalHall => 0.7,
            Self::ChamberHall => 0.6,
            Self::SymphonyHall => 0.5,
            Self::GrandHall => 0.45,
            Self::Cathedral => 0.3,
            Self::Amphitheater => 0.2,
            Self::Studio => 0.8,
            Self::Custom => 0.5,
        }
    }
}

/// Concert hall model
#[derive(Debug, Clone)]
pub struct ConcertHall {
    pub hall_type: HallType,
    pub width: f32,
    pub depth: f32,
    pub height: f32,
    /// Reverb time in seconds
    pub reverb_time: f32,
    /// Early reflection delay (ms)
    pub early_reflection_delay: f32,
    /// Wall absorption coefficient (0 = reflective, 1 = absorptive)
    pub wall_absorption: f32,
    /// Air absorption coefficient
    pub air_absorption: f32,
    /// Stage dimensions (width, depth)
    pub stage_width: f32,
    pub stage_depth: f32,
}

impl ConcertHall {
    pub fn new(hall_type: HallType) -> Self {
        let (width, depth, height) = hall_type.dimensions();
        Self {
            hall_type,
            width,
            depth,
            height,
            reverb_time: hall_type.reverb_time(),
            early_reflection_delay: 20.0 + width * 0.5,
            wall_absorption: 0.3,
            air_absorption: 0.001,
            stage_width: width * 0.6,
            stage_depth: depth * 0.25,
        }
    }

    /// Symphony hall preset
    pub fn symphony_hall() -> Self {
        Self::new(HallType::SymphonyHall)
    }

    /// Chamber music hall preset
    pub fn chamber_hall() -> Self {
        Self::new(HallType::ChamberHall)
    }

    /// Recording studio preset
    pub fn studio() -> Self {
        Self::new(HallType::Studio)
    }

    /// Calculate distance attenuation
    pub fn distance_attenuation(&self, distance: f32) -> f32 {
        if distance < 1.0 {
            return 1.0;
        }
        // Inverse square law with air absorption
        let geometric = 1.0 / distance;
        let air_loss = (-self.air_absorption * distance).exp();
        geometric * air_loss
    }

    /// Calculate early reflection gain based on position
    pub fn early_reflection_gain(&self, source: &Position, listener: &Position) -> f32 {
        let _direct_distance = source.distance_to(listener);

        // Simplified: reflections from walls add energy
        let reflection_density = self.hall_type.reflection_density();
        let wall_distance = (self.width / 2.0 - source.x.abs()).min(self.depth - source.y);

        if wall_distance < 1.0 {
            reflection_density * 0.5 // Close to wall, more reflections
        } else {
            reflection_density * 0.3 * (1.0 - self.wall_absorption)
        }
    }
}

impl Default for ConcertHall {
    fn default() -> Self {
        Self::symphony_hall()
    }
}

/// Orchestra section types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrchestraSection {
    // Strings
    Violin1,
    Violin2,
    Viola,
    Cello,
    DoubleBass,

    // Woodwinds
    Flute,
    Oboe,
    Clarinet,
    Bassoon,

    // Brass
    FrenchHorn,
    Trumpet,
    Trombone,
    Tuba,

    // Percussion
    Timpani,
    Percussion,

    // Choir
    Soprano,
    Alto,
    Tenor,
    Bass,

    // Special
    Harp,
    Piano,
    Organ,
    Soloist,
}

impl OrchestraSection {
    /// Get typical section size
    pub fn typical_size(&self) -> usize {
        match self {
            Self::Violin1 => 16,
            Self::Violin2 => 14,
            Self::Viola => 12,
            Self::Cello => 10,
            Self::DoubleBass => 8,
            Self::Flute => 3,
            Self::Oboe => 3,
            Self::Clarinet => 3,
            Self::Bassoon => 3,
            Self::FrenchHorn => 4,
            Self::Trumpet => 3,
            Self::Trombone => 3,
            Self::Tuba => 1,
            Self::Timpani => 1,
            Self::Percussion => 4,
            Self::Soprano => 12,
            Self::Alto => 12,
            Self::Tenor => 10,
            Self::Bass => 10,
            Self::Harp => 1,
            Self::Piano => 1,
            Self::Organ => 1,
            Self::Soloist => 1,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Violin1 => "Violin I",
            Self::Violin2 => "Violin II",
            Self::Viola => "Viola",
            Self::Cello => "Cello",
            Self::DoubleBass => "Double Bass",
            Self::Flute => "Flute",
            Self::Oboe => "Oboe",
            Self::Clarinet => "Clarinet",
            Self::Bassoon => "Bassoon",
            Self::FrenchHorn => "French Horn",
            Self::Trumpet => "Trumpet",
            Self::Trombone => "Trombone",
            Self::Tuba => "Tuba",
            Self::Timpani => "Timpani",
            Self::Percussion => "Percussion",
            Self::Soprano => "Soprano",
            Self::Alto => "Alto",
            Self::Tenor => "Tenor",
            Self::Bass => "Bass (Choir)",
            Self::Harp => "Harp",
            Self::Piano => "Piano",
            Self::Organ => "Organ",
            Self::Soloist => "Soloist",
        }
    }

    /// Get section color for visualization (RGB)
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            // Strings - warm browns/reds
            Self::Violin1 => (180, 100, 60),
            Self::Violin2 => (160, 90, 55),
            Self::Viola => (140, 80, 50),
            Self::Cello => (120, 70, 45),
            Self::DoubleBass => (100, 60, 40),

            // Woodwinds - greens/teals
            Self::Flute => (150, 200, 180),
            Self::Oboe => (100, 180, 140),
            Self::Clarinet => (80, 160, 120),
            Self::Bassoon => (60, 140, 100),

            // Brass - golds/yellows
            Self::FrenchHorn => (220, 180, 80),
            Self::Trumpet => (240, 200, 60),
            Self::Trombone => (200, 160, 50),
            Self::Tuba => (180, 140, 40),

            // Percussion - grays/silvers
            Self::Timpani => (120, 120, 130),
            Self::Percussion => (100, 100, 110),

            // Choir - purples/blues
            Self::Soprano => (200, 150, 220),
            Self::Alto => (180, 130, 200),
            Self::Tenor => (160, 110, 180),
            Self::Bass => (140, 90, 160),

            // Special - distinct colors
            Self::Harp => (220, 200, 160),
            Self::Piano => (40, 40, 40),
            Self::Organ => (80, 80, 100),
            Self::Soloist => (255, 100, 100),
        }
    }
}

/// Individual musician in the orchestra
#[derive(Debug, Clone)]
pub struct Musician {
    /// Unique identifier
    pub id: u32,
    /// Section this musician belongs to
    pub section: OrchestraSection,
    /// Position in the concert hall
    pub position: Position,
    /// Individual volume (0.0 - 1.0)
    pub volume: f32,
    /// Muted state
    pub muted: bool,
    /// Solo state
    pub solo: bool,
    /// Chair number within section (1 = principal)
    pub chair: u8,
}

impl Musician {
    pub fn new(id: u32, section: OrchestraSection, position: Position, chair: u8) -> Self {
        Self {
            id,
            section,
            position,
            volume: 1.0,
            muted: false,
            solo: false,
            chair,
        }
    }
}

/// Seating arrangement presets
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SeatingArrangement {
    /// Traditional American seating (Violin II opposite Violin I)
    American,
    /// German/European seating (Violin I and II on left)
    German,
    /// Stokowski arrangement (antiphonal strings)
    Stokowski,
    /// Baroque/period instrument arrangement
    Baroque,
    /// Modern flexible arrangement
    Modern,
    /// Custom user arrangement
    Custom,
}

impl SeatingArrangement {
    /// Get default positions for a section
    pub fn section_center(&self, section: OrchestraSection) -> Position {
        match self {
            Self::American => Self::american_position(section),
            Self::German => Self::german_position(section),
            Self::Stokowski => Self::stokowski_position(section),
            Self::Baroque => Self::baroque_position(section),
            Self::Modern | Self::Custom => Self::american_position(section),
        }
    }

    fn american_position(section: OrchestraSection) -> Position {
        match section {
            // Strings - front arc
            OrchestraSection::Violin1 => Position::new(-6.0, 0.0, 0.0),
            OrchestraSection::Violin2 => Position::new(6.0, 0.0, 0.0),
            OrchestraSection::Viola => Position::new(3.0, 2.0, 0.0),
            OrchestraSection::Cello => Position::new(-3.0, 2.0, 0.0),
            OrchestraSection::DoubleBass => Position::new(-8.0, 3.0, 0.0),

            // Woodwinds - second row center
            OrchestraSection::Flute => Position::new(-2.0, 4.0, 0.3),
            OrchestraSection::Oboe => Position::new(0.0, 4.0, 0.3),
            OrchestraSection::Clarinet => Position::new(2.0, 4.5, 0.3),
            OrchestraSection::Bassoon => Position::new(4.0, 4.5, 0.3),

            // Brass - back row
            OrchestraSection::FrenchHorn => Position::new(-4.0, 6.0, 0.5),
            OrchestraSection::Trumpet => Position::new(0.0, 7.0, 0.5),
            OrchestraSection::Trombone => Position::new(3.0, 7.0, 0.5),
            OrchestraSection::Tuba => Position::new(6.0, 7.0, 0.5),

            // Percussion - far back
            OrchestraSection::Timpani => Position::new(-6.0, 8.0, 0.3),
            OrchestraSection::Percussion => Position::new(0.0, 9.0, 0.5),

            // Choir - behind orchestra or risers
            OrchestraSection::Soprano => Position::new(-4.0, 10.0, 1.0),
            OrchestraSection::Alto => Position::new(-1.0, 10.0, 1.0),
            OrchestraSection::Tenor => Position::new(1.0, 10.0, 1.0),
            OrchestraSection::Bass => Position::new(4.0, 10.0, 1.0),

            // Special instruments
            OrchestraSection::Harp => Position::new(-10.0, 3.0, 0.0),
            OrchestraSection::Piano => Position::new(-8.0, 1.0, 0.0),
            OrchestraSection::Organ => Position::new(0.0, 12.0, 2.0),
            OrchestraSection::Soloist => Position::new(0.0, -1.0, 0.0),
        }
    }

    fn german_position(section: OrchestraSection) -> Position {
        match section {
            // German: Violin I and II both on left, Cello/Bass on right
            OrchestraSection::Violin1 => Position::new(-6.0, 0.0, 0.0),
            OrchestraSection::Violin2 => Position::new(-4.0, 2.0, 0.0),
            OrchestraSection::Viola => Position::new(4.0, 2.0, 0.0),
            OrchestraSection::Cello => Position::new(6.0, 0.0, 0.0),
            OrchestraSection::DoubleBass => Position::new(8.0, 2.0, 0.0),
            _ => Self::american_position(section),
        }
    }

    fn stokowski_position(section: OrchestraSection) -> Position {
        match section {
            // Stokowski: Antiphonal violins, basses center back
            OrchestraSection::Violin1 => Position::new(-7.0, 0.0, 0.0),
            OrchestraSection::Violin2 => Position::new(7.0, 0.0, 0.0),
            OrchestraSection::Viola => Position::new(-3.0, 2.0, 0.0),
            OrchestraSection::Cello => Position::new(3.0, 2.0, 0.0),
            OrchestraSection::DoubleBass => Position::new(0.0, 5.0, 0.0),
            _ => Self::american_position(section),
        }
    }

    fn baroque_position(section: OrchestraSection) -> Position {
        match section {
            // Baroque: Smaller, more intimate arrangement
            OrchestraSection::Violin1 => Position::new(-4.0, 0.0, 0.0),
            OrchestraSection::Violin2 => Position::new(4.0, 0.0, 0.0),
            OrchestraSection::Viola => Position::new(2.0, 1.5, 0.0),
            OrchestraSection::Cello => Position::new(-2.0, 1.5, 0.0),
            OrchestraSection::DoubleBass => Position::new(-4.0, 2.0, 0.0),
            _ => Self::american_position(section),
        }
    }
}

/// Spatial audio processor for a single source
#[derive(Debug, Clone)]
pub struct SpatialProcessor {
    /// Source position
    pub position: Position,
    /// Listener reference
    listener: Listener,
    /// Calculated pan (-1 to 1)
    pan: f32,
    /// Calculated distance attenuation
    distance_gain: f32,
    /// Early reflection delay samples
    early_delay_samples: usize,
    /// Early reflection gain
    early_gain: f32,
    /// Delay buffer for early reflections
    delay_buffer: Vec<f32>,
    delay_write_pos: usize,
    /// Sample rate
    sample_rate: f32,
}

impl SpatialProcessor {
    pub fn new(position: Position, listener: Listener, hall: &ConcertHall, sample_rate: f32) -> Self {
        let distance = position.distance_to(&listener.position);
        let angle = position.angle_from(&listener.position);

        // Calculate pan based on angle
        let pan = (angle - listener.rotation).sin().clamp(-1.0, 1.0);

        // Calculate distance attenuation
        let distance_gain = hall.distance_attenuation(distance);

        // Early reflection delay based on wall distance
        let early_delay_ms = hall.early_reflection_delay + distance * 2.0;
        let early_delay_samples = ((early_delay_ms / 1000.0) * sample_rate) as usize;

        // Early reflection gain
        let early_gain = hall.early_reflection_gain(&position, &listener.position);

        // Allocate delay buffer for reflections
        let max_delay = (sample_rate * 0.1) as usize; // 100ms max

        Self {
            position,
            listener,
            pan,
            distance_gain,
            early_delay_samples: early_delay_samples.min(max_delay - 1),
            early_gain,
            delay_buffer: vec![0.0; max_delay],
            delay_write_pos: 0,
            sample_rate,
        }
    }

    /// Update position (e.g., when musician moves)
    pub fn set_position(&mut self, position: Position, hall: &ConcertHall) {
        self.position = position;
        let distance = position.distance_to(&self.listener.position);
        let angle = position.angle_from(&self.listener.position);

        self.pan = (angle - self.listener.rotation).sin().clamp(-1.0, 1.0);
        self.distance_gain = hall.distance_attenuation(distance);
        self.early_gain = hall.early_reflection_gain(&position, &self.listener.position);
    }

    /// Process mono input to stereo output with spatialization
    pub fn process(&mut self, input: f32) -> (f32, f32) {
        // Apply distance attenuation
        let direct = input * self.distance_gain;

        // Write to delay buffer
        self.delay_buffer[self.delay_write_pos] = input;
        self.delay_write_pos = (self.delay_write_pos + 1) % self.delay_buffer.len();

        // Read early reflection
        let read_pos = (self.delay_write_pos + self.delay_buffer.len() - self.early_delay_samples)
            % self.delay_buffer.len();
        let early = self.delay_buffer[read_pos] * self.early_gain * self.distance_gain;

        // Combine direct and early reflection
        let combined = direct + early;

        // Apply panning (constant power pan law)
        let pan_angle = (self.pan + 1.0) * 0.25 * PI; // 0 to PI/2
        let left = combined * pan_angle.cos();
        let right = combined * pan_angle.sin();

        (left, right)
    }

    /// Get current pan value for visualization
    pub fn get_pan(&self) -> f32 {
        self.pan
    }

    /// Get current distance for visualization
    pub fn get_distance(&self) -> f32 {
        self.position.distance_to(&self.listener.position)
    }
}

/// Section mixer - mixes multiple musicians in a section
#[derive(Debug, Clone)]
pub struct SectionMixer {
    pub section: OrchestraSection,
    pub musicians: Vec<Musician>,
    pub spatial_processors: Vec<SpatialProcessor>,
    /// Section volume (0.0 - 1.0)
    pub volume: f32,
    /// Section pan adjustment (-1.0 to 1.0)
    pub pan: f32,
    /// Section muted
    pub muted: bool,
    /// Section solo
    pub solo: bool,
}

impl SectionMixer {
    pub fn new(
        section: OrchestraSection,
        arrangement: SeatingArrangement,
        count: usize,
        hall: &ConcertHall,
        listener: Listener,
        sample_rate: f32,
    ) -> Self {
        let center = arrangement.section_center(section);
        let mut musicians = Vec::with_capacity(count);
        let mut spatial_processors = Vec::with_capacity(count);

        // Spread musicians around the section center
        let spread = match section {
            OrchestraSection::Violin1 | OrchestraSection::Violin2 => 4.0,
            OrchestraSection::Viola | OrchestraSection::Cello => 3.0,
            OrchestraSection::DoubleBass => 3.0,
            OrchestraSection::Soprano | OrchestraSection::Alto |
            OrchestraSection::Tenor | OrchestraSection::Bass => 3.0,
            _ => 1.5,
        };

        for i in 0..count {
            let offset_x = if count > 1 {
                (i as f32 / (count - 1) as f32 - 0.5) * spread
            } else {
                0.0
            };
            let offset_y = (i as f32 * 0.15).sin() * 0.5; // Slight staggering

            let pos = Position::new(
                center.x + offset_x,
                center.y + offset_y,
                center.z,
            );

            let id = (section as u32) * 1000 + i as u32;
            musicians.push(Musician::new(id, section, pos, (i + 1) as u8));
            spatial_processors.push(SpatialProcessor::new(pos, listener, hall, sample_rate));
        }

        Self {
            section,
            musicians,
            spatial_processors,
            volume: 1.0,
            pan: 0.0,
            muted: false,
            solo: false,
        }
    }

    /// Move a musician to a new position
    pub fn move_musician(&mut self, index: usize, position: Position, hall: &ConcertHall) {
        if index < self.musicians.len() {
            self.musicians[index].position = position;
            self.spatial_processors[index].set_position(position, hall);
        }
    }

    /// Process audio from section (mono inputs for each musician)
    pub fn process(&mut self, inputs: &[f32]) -> (f32, f32) {
        if self.muted {
            return (0.0, 0.0);
        }

        let mut left = 0.0;
        let mut right = 0.0;

        for (i, input) in inputs.iter().enumerate() {
            if i >= self.spatial_processors.len() {
                break;
            }

            if self.musicians[i].muted {
                continue;
            }

            let musician_vol = self.musicians[i].volume;
            let (l, r) = self.spatial_processors[i].process(*input * musician_vol);
            left += l;
            right += r;
        }

        // Apply section volume and pan
        left *= self.volume * (1.0 + self.pan.min(0.0));
        right *= self.volume * (1.0 - self.pan.max(0.0));

        (left, right)
    }

    /// Get visualization data for this section
    pub fn get_visualization(&self) -> SectionVisualization {
        let positions: Vec<_> = self.musicians.iter().map(|m| m.position).collect();
        let (r, g, b) = self.section.color();

        SectionVisualization {
            section: self.section,
            positions,
            color: (r, g, b),
            volume: self.volume,
            muted: self.muted,
            solo: self.solo,
        }
    }
}

/// Full orchestra mixer
#[derive(Debug)]
pub struct Orchestra {
    /// Concert hall
    pub hall: ConcertHall,
    /// Listener position
    pub listener: Listener,
    /// Seating arrangement
    pub arrangement: SeatingArrangement,
    /// Section mixers
    sections: Vec<SectionMixer>,
    /// Master volume
    pub master_volume: f32,
    /// Sample rate
    sample_rate: f32,
    /// Any section in solo mode?
    has_solo: bool,
}

impl Orchestra {
    pub fn new(hall: ConcertHall, arrangement: SeatingArrangement, sample_rate: f32) -> Self {
        Self {
            hall,
            listener: Listener::default(),
            arrangement,
            sections: Vec::new(),
            master_volume: 1.0,
            sample_rate,
            has_solo: false,
        }
    }

    /// Create a full symphony orchestra
    pub fn symphony(sample_rate: f32) -> Self {
        let hall = ConcertHall::symphony_hall();
        let arrangement = SeatingArrangement::American;

        let mut orchestra = Self::new(hall, arrangement, sample_rate);

        // Add all standard symphony sections
        let sections = [
            (OrchestraSection::Violin1, 16),
            (OrchestraSection::Violin2, 14),
            (OrchestraSection::Viola, 12),
            (OrchestraSection::Cello, 10),
            (OrchestraSection::DoubleBass, 8),
            (OrchestraSection::Flute, 3),
            (OrchestraSection::Oboe, 3),
            (OrchestraSection::Clarinet, 3),
            (OrchestraSection::Bassoon, 3),
            (OrchestraSection::FrenchHorn, 4),
            (OrchestraSection::Trumpet, 3),
            (OrchestraSection::Trombone, 3),
            (OrchestraSection::Tuba, 1),
            (OrchestraSection::Timpani, 1),
            (OrchestraSection::Percussion, 4),
            (OrchestraSection::Harp, 1),
        ];

        for (section, count) in sections {
            orchestra.add_section(section, count);
        }

        orchestra
    }

    /// Create a chamber orchestra
    pub fn chamber(sample_rate: f32) -> Self {
        let hall = ConcertHall::chamber_hall();
        let arrangement = SeatingArrangement::Baroque;
        let mut orchestra = Self::new(hall.clone(), arrangement, sample_rate);

        let sections = [
            (OrchestraSection::Violin1, 6),
            (OrchestraSection::Violin2, 5),
            (OrchestraSection::Viola, 4),
            (OrchestraSection::Cello, 3),
            (OrchestraSection::DoubleBass, 2),
            (OrchestraSection::Flute, 1),
            (OrchestraSection::Oboe, 2),
            (OrchestraSection::Bassoon, 1),
            (OrchestraSection::FrenchHorn, 2),
        ];

        for (section, count) in sections {
            orchestra.add_section(section, count);
        }

        orchestra
    }

    /// Create orchestra with choir
    pub fn symphony_with_choir(sample_rate: f32) -> Self {
        let mut orchestra = Self::symphony(sample_rate);

        orchestra.add_section(OrchestraSection::Soprano, 12);
        orchestra.add_section(OrchestraSection::Alto, 12);
        orchestra.add_section(OrchestraSection::Tenor, 10);
        orchestra.add_section(OrchestraSection::Bass, 10);

        orchestra
    }

    /// Add a section to the orchestra
    pub fn add_section(&mut self, section: OrchestraSection, count: usize) {
        let mixer = SectionMixer::new(
            section,
            self.arrangement,
            count,
            &self.hall,
            self.listener,
            self.sample_rate,
        );
        self.sections.push(mixer);
    }

    /// Get a section mixer by type
    pub fn get_section(&self, section: OrchestraSection) -> Option<&SectionMixer> {
        self.sections.iter().find(|s| s.section == section)
    }

    /// Get a mutable section mixer by type
    pub fn get_section_mut(&mut self, section: OrchestraSection) -> Option<&mut SectionMixer> {
        self.sections.iter_mut().find(|s| s.section == section)
    }

    /// Set listener position
    pub fn set_listener(&mut self, listener: Listener) {
        self.listener = listener;
        // Update all spatial processors
        for section in &mut self.sections {
            for (i, processor) in section.spatial_processors.iter_mut().enumerate() {
                processor.listener = listener;
                processor.set_position(section.musicians[i].position, &self.hall);
            }
        }
    }

    /// Move a musician
    pub fn move_musician(&mut self, section: OrchestraSection, index: usize, position: Position) {
        // Clone hall to avoid borrow conflict
        let hall = self.hall.clone();
        if let Some(sec) = self.get_section_mut(section) {
            sec.move_musician(index, position, &hall);
        }
    }

    /// Solo a section
    pub fn solo_section(&mut self, section: OrchestraSection, solo: bool) {
        if let Some(sec) = self.get_section_mut(section) {
            sec.solo = solo;
        }
        self.has_solo = self.sections.iter().any(|s| s.solo);
    }

    /// Mute a section
    pub fn mute_section(&mut self, section: OrchestraSection, mute: bool) {
        if let Some(sec) = self.get_section_mut(section) {
            sec.muted = mute;
        }
    }

    /// Set section volume
    pub fn set_section_volume(&mut self, section: OrchestraSection, volume: f32) {
        if let Some(sec) = self.get_section_mut(section) {
            sec.volume = volume.clamp(0.0, 2.0);
        }
    }

    /// Process all sections
    /// inputs: HashMap<OrchestraSection, Vec<f32>> - audio for each musician in each section
    pub fn process(&mut self, section_inputs: &[(OrchestraSection, Vec<f32>)]) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;

        for (section_type, inputs) in section_inputs {
            if let Some(section) = self.sections.iter_mut().find(|s| s.section == *section_type) {
                // Skip if solo mode is active and this section isn't solo'd
                if self.has_solo && !section.solo {
                    continue;
                }

                let (l, r) = section.process(inputs);
                left += l;
                right += r;
            }
        }

        // Apply master volume
        (left * self.master_volume, right * self.master_volume)
    }

    /// Get full visualization data
    pub fn get_visualization(&self) -> OrchestraVisualization {
        let sections: Vec<_> = self.sections.iter().map(|s| s.get_visualization()).collect();

        OrchestraVisualization {
            hall_width: self.hall.width,
            hall_depth: self.hall.depth,
            stage_width: self.hall.stage_width,
            stage_depth: self.hall.stage_depth,
            listener_position: self.listener.position,
            sections,
        }
    }

    /// Get list of all sections
    pub fn list_sections(&self) -> Vec<OrchestraSection> {
        self.sections.iter().map(|s| s.section).collect()
    }

    /// Get total musician count
    pub fn musician_count(&self) -> usize {
        self.sections.iter().map(|s| s.musicians.len()).sum()
    }
}

/// Visualization data for a section
#[derive(Debug, Clone)]
pub struct SectionVisualization {
    pub section: OrchestraSection,
    pub positions: Vec<Position>,
    pub color: (u8, u8, u8),
    pub volume: f32,
    pub muted: bool,
    pub solo: bool,
}

/// Full orchestra visualization data
#[derive(Debug, Clone)]
pub struct OrchestraVisualization {
    pub hall_width: f32,
    pub hall_depth: f32,
    pub stage_width: f32,
    pub stage_depth: f32,
    pub listener_position: Position,
    pub sections: Vec<SectionVisualization>,
}

impl OrchestraVisualization {
    /// Convert position to normalized coordinates (0-1) for rendering
    pub fn position_to_normalized(&self, pos: &Position) -> (f32, f32) {
        let x = (pos.x / self.hall_width + 0.5).clamp(0.0, 1.0);
        let y = (pos.y / self.hall_depth + 0.1).clamp(0.0, 1.0);
        (x, y)
    }

    /// Get all musician positions as normalized coordinates with section info
    pub fn get_all_musicians(&self) -> Vec<(f32, f32, OrchestraSection, (u8, u8, u8))> {
        let mut result = Vec::new();

        for section in &self.sections {
            for pos in &section.positions {
                let (x, y) = self.position_to_normalized(pos);
                result.push((x, y, section.section, section.color));
            }
        }

        result
    }

    /// Get JSON-serializable representation for web UI
    pub fn to_json_value(&self) -> String {
        let mut json = String::from("{");

        json.push_str(&format!(
            "\"hall\":{{\"width\":{},\"depth\":{},\"stageWidth\":{},\"stageDepth\":{}}},",
            self.hall_width, self.hall_depth, self.stage_width, self.stage_depth
        ));

        json.push_str(&format!(
            "\"listener\":{{\"x\":{},\"y\":{}}},",
            self.listener_position.x, self.listener_position.y
        ));

        json.push_str("\"sections\":[");
        for (i, section) in self.sections.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&format!(
                "{{\"name\":\"{}\",\"color\":[{},{},{}],\"volume\":{},\"muted\":{},\"solo\":{},\"positions\":[",
                section.section.display_name(),
                section.color.0, section.color.1, section.color.2,
                section.volume,
                section.muted,
                section.solo
            ));
            for (j, pos) in section.positions.iter().enumerate() {
                if j > 0 {
                    json.push(',');
                }
                json.push_str(&format!("{{\"x\":{},\"y\":{},\"z\":{}}}", pos.x, pos.y, pos.z));
            }
            json.push_str("]}");
        }
        json.push_str("]}");

        json
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position() {
        let p1 = Position::new(0.0, 0.0, 0.0);
        let p2 = Position::new(3.0, 4.0, 0.0);

        assert!((p1.distance_to(&p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_hall_types() {
        let halls = [
            HallType::RecitalHall,
            HallType::ChamberHall,
            HallType::SymphonyHall,
            HallType::GrandHall,
            HallType::Cathedral,
        ];

        for hall_type in halls {
            let hall = ConcertHall::new(hall_type);
            assert!(hall.width > 0.0);
            assert!(hall.reverb_time > 0.0);
        }
    }

    #[test]
    fn test_orchestra_creation() {
        let sample_rate = 44100.0;
        let orchestra = Orchestra::symphony(sample_rate);

        // Should have standard sections
        assert!(orchestra.get_section(OrchestraSection::Violin1).is_some());
        assert!(orchestra.get_section(OrchestraSection::Trumpet).is_some());
        assert!(orchestra.get_section(OrchestraSection::Timpani).is_some());

        // Should not have choir by default
        assert!(orchestra.get_section(OrchestraSection::Soprano).is_none());

        // Total musician count
        assert!(orchestra.musician_count() > 50);
    }

    #[test]
    fn test_orchestra_with_choir() {
        let sample_rate = 44100.0;
        let orchestra = Orchestra::symphony_with_choir(sample_rate);

        // Should have choir sections
        assert!(orchestra.get_section(OrchestraSection::Soprano).is_some());
        assert!(orchestra.get_section(OrchestraSection::Alto).is_some());
        assert!(orchestra.get_section(OrchestraSection::Tenor).is_some());
        assert!(orchestra.get_section(OrchestraSection::Bass).is_some());
    }

    #[test]
    fn test_chamber_orchestra() {
        let sample_rate = 44100.0;
        let orchestra = Orchestra::chamber(sample_rate);

        // Smaller sections
        let v1 = orchestra.get_section(OrchestraSection::Violin1).unwrap();
        assert!(v1.musicians.len() < 10);
    }

    #[test]
    fn test_spatial_processing() {
        let sample_rate = 44100.0;
        let hall = ConcertHall::symphony_hall();
        let listener = Listener::default();

        // Left side position
        let left_pos = Position::new(-5.0, 2.0, 0.0);
        let mut left_proc = SpatialProcessor::new(left_pos, listener, &hall, sample_rate);

        // Right side position
        let right_pos = Position::new(5.0, 2.0, 0.0);
        let mut right_proc = SpatialProcessor::new(right_pos, listener, &hall, sample_rate);

        // Process test signal
        let (l1, r1) = left_proc.process(1.0);
        let (l2, r2) = right_proc.process(1.0);

        // Left source should have more left channel
        assert!(l1 > r1);
        // Right source should have more right channel
        assert!(r2 > l2);
    }

    #[test]
    fn test_visualization() {
        let sample_rate = 44100.0;
        let orchestra = Orchestra::symphony(sample_rate);
        let viz = orchestra.get_visualization();

        assert!(viz.sections.len() > 0);
        assert!(viz.hall_width > 0.0);

        // Check JSON generation doesn't panic
        let json = viz.to_json_value();
        assert!(json.contains("sections"));
        assert!(json.contains("Violin I"));
    }

    #[test]
    fn test_section_colors() {
        let sections = [
            OrchestraSection::Violin1,
            OrchestraSection::Trumpet,
            OrchestraSection::Flute,
            OrchestraSection::Timpani,
        ];

        for section in sections {
            let (r, g, b) = section.color();
            // Colors should be valid RGB
            assert!(r > 0 || g > 0 || b > 0);
        }
    }

    #[test]
    fn test_musician_movement() {
        let sample_rate = 44100.0;
        let mut orchestra = Orchestra::symphony(sample_rate);

        let original_pos = orchestra
            .get_section(OrchestraSection::Violin1)
            .unwrap()
            .musicians[0]
            .position;

        // Move to a very different position
        let new_pos = Position::new(5.0, 8.0, 1.0);
        orchestra.move_musician(OrchestraSection::Violin1, 0, new_pos);

        let updated_pos = orchestra
            .get_section(OrchestraSection::Violin1)
            .unwrap()
            .musicians[0]
            .position;

        // Position should be updated to new position
        assert!((updated_pos.x - new_pos.x).abs() < 0.001);
        assert!((updated_pos.y - new_pos.y).abs() < 0.001);
        assert!((updated_pos.z - new_pos.z).abs() < 0.001);
    }

    #[test]
    fn test_seating_arrangements() {
        let arrangements = [
            SeatingArrangement::American,
            SeatingArrangement::German,
            SeatingArrangement::Stokowski,
            SeatingArrangement::Baroque,
        ];

        for arr in arrangements {
            let v1_pos = arr.section_center(OrchestraSection::Violin1);
            let v2_pos = arr.section_center(OrchestraSection::Violin2);

            // Violin sections should be in different positions
            assert!(v1_pos.distance_to(&v2_pos) > 1.0);
        }
    }

    #[test]
    fn test_section_solo_mute() {
        let sample_rate = 44100.0;
        let mut orchestra = Orchestra::symphony(sample_rate);

        // Solo strings
        orchestra.solo_section(OrchestraSection::Violin1, true);
        assert!(orchestra.get_section(OrchestraSection::Violin1).unwrap().solo);
        assert!(orchestra.has_solo);

        // Mute brass
        orchestra.mute_section(OrchestraSection::Trumpet, true);
        assert!(orchestra.get_section(OrchestraSection::Trumpet).unwrap().muted);
    }
}

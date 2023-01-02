#![crate_name = "syarm_lib"]
//! # SyArm library
//! 
//! Control and calculation library for the SyArm robot

// Module decleration
    mod pvec;
    mod types;
    pub mod interpreter;
//

// Imports
use std::{fs, f32::consts::PI, vec};
use serde::{Serialize, Deserialize};

use stepper_lib::{
    Component,
    ctrl::StepperCtrl, 
    comp::{Cylinder, GearBearing, CylinderTriangle, Tool, NoTool, PencilTool}, 
    data::StepperData, 
    math::{inertia_point, inertia_rod_constr, forces_segment, inertia_to_mass, forces_joint}
};

// Local imports
use pvec::PVec3;
pub use types::*;
pub use interpreter::init_interpreter;
pub use stepper_lib::gcode::Interpreter;

// Constants
/// Gravitational acceleration as vector
const G : Vec3 = Vec3 { x: 0.0, y: 0.0, z: -9.805 };

/// Set of zero value forces
pub const FORCES_ZERO : Forces = Forces(0.0, 0.0, 0.0, 0.0);
/// Set of zero value inertias
pub const INERTIAS_ZERO : Inertias = Inertias(0.0, 0.0, 0.0, 0.0);

// Structures
    /// ### Constants
    /// All the constats required for the calculation of the syarm \
    /// JSON I/O is enabled via the `serde_json` library
    #[derive(Serialize, Deserialize)]
    pub struct Constants 
    {
        // Circuit
        /// Voltage supplied to the motors in Volts
        pub u : f32,                    

        /// Direction ctrl pin for the base controller
        pub pin_dir_b : u16,         
        /// Step ctrl pin for the base controller
        pub pin_step_b : u16,          
        /// Measure pin for the base controller
        pub pin_meas_b : u16,

        /// Direction ctrl pin for the first cylinder
        pub pin_dir_1 : u16, 
        /// Step ctrl pin for the first cylinder
        pub pin_step_1 : u16, 
        /// Measure pin for the base controller
        pub pin_meas_1 : u16,

        /// Direction ctrl pin for the second cylinder
        pub pin_dir_2 : u16,
        /// Step ctrl pin for the second cylinder
        pub pin_step_2 : u16,
        /// Measure pin for the second cylinder
        pub pin_meas_2 : u16,

        /// Direction ctrl pin for the third cylinder
        pub pin_dir_3 : u16,
        /// Step ctrl pin for the second cylinder
        pub pin_step_3 : u16,
        /// Measure pin for the third cylinder
        pub pin_meas_3 : u16,

        // Measured
        /// Set value for the base joint when measured in radians
        pub meas_b : f32,
        /// Set value for the first cylinder when measured in mm
        pub meas_a1 : f32,
        /// Set value for the second cylinder when measured in mm
        pub meas_a2 : f32,
        /// Set value for the third joint when measured in radians
        pub meas_a3 : f32,

        // Construction
        /// Base vector when in base position, x, y and z lengths in mm
        pub a_b : PVec3,

        /// Length of the first arm segment in mm
        pub l_a1 : f32,
        /// Length of the second arm segment in mm
        pub l_a2 : f32,
        /// Length of the third arm segment in mm
        pub l_a3 : f32,

        /// Length of a-segment of first cylinder triangle in mm
        pub l_c1a : f32,
        /// Length of b-segment of first cylinder triangle in mm
        pub l_c1b : f32, 
        /// Length of a-segment of second cylinder triangle in mm
        pub l_c2a : f32,
        /// Length of b-segment of second cylinder triangle in mm
        pub l_c2b : f32,

        /// Additional angle of a-segment of first cylinder triangle in mm
        pub delta_1a : f32,
        /// Additional angle of b-segment of first cylinder triangle in mm
        pub delta_1b : f32,
        /// Additional angle of a-segment of second cylinder triangle in mm
        pub delta_2a : f32,
        /// Additional angle of b-segment of second cylinder triangle in mm
        pub delta_2b : f32,

        /// Minimum base joint angle in radians
        pub phib_min : f32, 
        /// Maximum base joint angle in radians
        pub phib_max : f32,
        /// Angular speed of base joint in radians per second
        pub omega_b : f32,
        /// Gear ratio of base joint
        pub ratio_b : f32,

        /// Maximum extension of first cylinder in mm
        pub c1_max : f32,
        /// Linear velocity of first cylinder in mm per second
        pub c1_v : f32,
        /// Spindle pitch in mm per radians
        pub ratio_1 : f32,

        /// Maximum extension of second cylinder in mm
        pub c2_max : f32,
        /// Linear velocity of second cylinder in mm per second
        pub c2_v : f32,
        /// Spindle pitch in mm per radians
        pub ratio_2 : f32,

        /// Maximum base join angle in radians
        pub phi3_max : f32,
        /// Angular speed of base joint in radians per second
        pub omega_3 : f32,
        /// Gear ratio of third joint
        pub ratio_3 : f32,

        // Load calculation
        /// Mass of base in kg
        pub m_b : f32,
        /// Mass of first arm segment in kg
        pub m_a1 : f32,
        /// Mass of second arm segment in kg
        pub m_a2 : f32,
        /// Mass of thrid arm segment in kg
        pub m_a3 : f32,

        /// Safety factor for calculations
        pub sf : f32
    }

    pub struct Variables
    {
        pub load : f32,

        pub dec_angle : f32,
        pub point : Vec3
    }

    /// Calculation and control struct for the SyArm robot
    pub struct SyArm
    {
        // Values
        pub cons : Constants,
        pub vars : Variables,
        pub tools : Vec<Box<dyn Tool + std::marker::Send>>,

        // Controls
        pub ctrl_base : GearBearing,
        pub ctrl_a1 : CylinderTriangle,
        pub ctrl_a2 : CylinderTriangle,
        pub ctrl_a3 : GearBearing,

        tool_id : usize
    }
//

/// Returns the angle of a vector to the X-Axis viewed from the Z-Axis
pub fn top_down_angle(point : Vec3) -> f32 {
    Vec3::new(point.x, point.y, 0.0).angle_between(Vec3::X)
}

fn _angle_to_deg(angles : Phis) -> Phis {
    Phis( 
        angles.0 * 180.0 / PI,
        angles.1 * 180.0 / PI,
        angles.2 * 180.0 / PI,
        angles.3 * 180.0 / PI
    )
}

pub fn law_of_cosines(a : f32, b : f32, c : f32) -> f32 {
    ((a.powi(2) + b.powi(2) - c.powi(2)) / 2.0 / a / b).acos()
}

impl SyArm
{
    // IO
        /// Creates a new syarm instance by a constants table
        pub fn from_const(cons : Constants) -> Self {
            Self { 
                tools: vec![ 
                    Box::new(NoTool::new()),
                    Box::new(PencilTool::new(127.0, 0.25))
                ],    
                ctrl_base: GearBearing { 
                    ctrl: StepperCtrl::new(
                        StepperData::mot_17he15_1504s(cons.u, cons.sf), cons.pin_dir_b, cons.pin_step_b
                    ), 
                    ratio: cons.ratio_b
                }, 
                ctrl_a1: CylinderTriangle::new(
                    Cylinder { 
                        ctrl: StepperCtrl::new(
                            StepperData::mot_17he15_1504s(cons.u, cons.sf), cons.pin_dir_1, cons.pin_step_1
                        ), 
                        rte_ratio: cons.ratio_1
                    },
                    cons.l_c1a, 
                    cons.l_c1b
                ), 
                ctrl_a2: CylinderTriangle::new(
                    Cylinder { 
                        ctrl: StepperCtrl::new(
                            StepperData::mot_17he15_1504s(cons.u, cons.sf), cons.pin_dir_2, cons.pin_step_2
                        ), 
                        rte_ratio: cons.ratio_2,
                    },
                    cons.l_c2a,
                    cons.l_c2b
                ), 
                ctrl_a3: GearBearing { 
                    ctrl: StepperCtrl::new(
                        StepperData::mot_17he15_1504s(cons.u, cons.sf), cons.pin_dir_3, cons.pin_step_3
                    ), 
                    ratio: cons.ratio_3
                },

                cons, 
                vars: Variables {
                    load: 0.0,

                    dec_angle: 0.0,
                    point: Vec3::ZERO
                },

                tool_id: 0
            }
        }
        
        pub fn get_cons_str(&self) -> String {
            serde_json::to_string(&self.cons).unwrap()
        }

        pub fn get_cons_str_pretty(&self) -> String {
            serde_json::to_string_pretty(&self.cons).unwrap()
        }

        /// Loads a new SyArm instance by creating a constants table out of the json file content at the given path
        pub fn load_json(path : &str) -> Self {
            let json_content  = fs::read_to_string(path).unwrap();
            return Self::from_const(serde_json::from_str(json_content.as_str()).unwrap());
        }

        // pub fn load_var(&mut self, path : &str) {
            
        // }

        // pub fn save_var(&self, path : &str) {
            
        // }

        /// Initializes measurement systems
        pub fn init_meas(&mut self) {
            self.ctrl_base.ctrl.init_meas(self.cons.pin_meas_b);
            self.ctrl_a1.cylinder.ctrl.init_meas(self.cons.pin_meas_1);
            self.ctrl_a2.cylinder.ctrl.init_meas(self.cons.pin_meas_2);
            self.ctrl_a3.ctrl.init_meas(self.cons.pin_meas_3);
        }
    // 

    // Tools
        pub fn get_tool(&self) -> Option<&Box<dyn Tool + std::marker::Send>> {
            self.tools.get(self.tool_id)
        }

        pub fn set_tool_id(&mut self, tool_id : usize) {
            self.tool_id = tool_id;
        }
    //

    // Angles
        // Phi: Used by calculation (rotation matrix)
        // Gamma: Used for controls (motor positioning)

        // Base
            /// Get the angles used by the calculations for the base
            pub fn phi_b(&self, gamma_b : f32) -> f32 {
                gamma_b
            }

            /// Get the angle used by the controls for the base
            pub fn gamma_b(&self, phi_b : f32) -> f32 {
                phi_b
            }
        //
        
        // First arm segment
            /// Get the angles used by the calculations for the first arm segment
            pub fn phi_a1(&self, gamma_a1 : f32) -> f32 {
                PI - gamma_a1 - self.cons.delta_1a - self.cons.delta_1b
            }

            /// Get the angle used by the controls for the first arm segment
            pub fn gamma_a1(&self, phi_a1 : f32) -> f32 {
                PI - phi_a1 - self.cons.delta_1a - self.cons.delta_1b
            }
        //
        
        // Second arm segment
            /// Get the angles used by the calculations for the second arm segment
            pub fn phi_a2(&self, gamma_a2 : f32) -> f32 {
                gamma_a2 + self.cons.delta_2a + self.cons.delta_2b - PI
            }

            /// Get the angle used by the controls for the second arm segment
            pub fn gamma_a2(&self, phi_a2 : f32) -> f32 {
                PI + phi_a2 - self.cons.delta_2a - self.cons.delta_2b
            }
        //
        
        // Third arm segment
            /// Get the angles used by the calculations for the third arm segment
            pub fn phi_a3(&self, gamma_a3 : f32) -> f32 {
                gamma_a3
            }

            /// Get the angle used by the controls for the third arm segment
            pub fn gamma_a3(&self, phi_a3 : f32) -> f32 {
                phi_a3
            }
        //

        /// Returns the four main angles used by the controls (gammas)
        pub fn get_all_gammas(&self) -> Gammas {
            Gammas( self.ctrl_base.get_dist(), self.ctrl_a1.get_dist(), self.ctrl_a2.get_dist(), self.ctrl_a3.get_dist() )
        }

        /// Converts gamma into phi angles
        pub fn gammas_for_phis(&self, phis : &Phis) -> Gammas {
            Gammas( self.gamma_b(phis.0), self.gamma_a1(phis.1), self.gamma_a2(phis.2), self.gamma_a3(phis.3) )
        } 

        /// Returns the four main angles used by the calculations (phis)
        pub fn get_all_phis(&self) -> Phis {
            Phis( 
                self.phi_b(self.ctrl_base.get_dist()), 
                self.phi_a1(self.ctrl_a1.get_dist()), 
                self.phi_a2(self.ctrl_a2.get_dist()), 
                self.phi_a3(self.ctrl_a3.get_dist())
            )
        }

        /// Converts phi into gamma angles
        pub fn phis_for_gammas(&self, gammas : &Gammas) -> Phis {
            Phis( self.phi_a1(gammas.0), self.phi_a1(gammas.1), self.phi_a2(gammas.2), self.phi_a3(gammas.3) )
        }

        pub fn valid_gammas(&self, gammas : Gammas) -> bool {
            let Gammas( g_b, g_1, g_2, g_3 ) = gammas;

            return 
                g_b.is_finite() & g_1.is_finite() & g_2.is_finite() & g_3.is_finite() & 
                (!self.ctrl_base.get_limit_dest(g_b).reached()) &
                (!self.ctrl_a1.get_limit_dest(g_1).reached())   &
                (!self.ctrl_a2.get_limit_dest(g_2).reached())   &
                (!self.ctrl_a3.get_limit_dest(g_3).reached())   
        }

        pub fn valid_gammas_det(&self, gammas : Gammas) -> (bool, bool, bool, bool) {
            let Gammas( g_b, g_1, g_2, g_3 ) = gammas;

            return (
                !self.ctrl_base.get_limit_dest(g_b).reached() & g_b.is_finite(),
                !self.ctrl_a1.get_limit_dest(g_1).reached() & g_1.is_finite(),
                !self.ctrl_a2.get_limit_dest(g_2).reached() & g_2.is_finite(),
                !self.ctrl_a3.get_limit_dest(g_3).reached() & g_3.is_finite()   
            )
        }

        pub fn valid_phis(&self, phis : &Phis) -> bool {
            self.valid_gammas(self.gammas_for_phis(phis))
        }
    //

    // Position calculation
        /// Get the vector of the decoration axis
        pub fn a_dec(&self) -> PVec3 {
            PVec3::new(Vec3::new(0.0, self.cons.l_a3, 0.0) + self.get_tool().unwrap().get_vec())
        }

        /// Returns the  points by the given  angles
        pub fn get_points_by_phis(&self, angles : &Phis) -> Points {
            let Vectors(a_b, a_1, a_2, a_3) = self.get_vectors_by_phis(angles);
            Points( 
                a_b,
                a_b + a_1,
                a_b + a_1 + a_2,
                a_b + a_1 + a_2 + a_3
            )
        }

        /// Get the (most relevant, characteristic) vectors of the robot by the  angles
        pub fn get_vectors_by_phis(&self, angles : &Phis) -> Vectors {
            // Rotation matrices used multiple times
            let base_rot = Mat3::from_rotation_z(angles.0);
            let a1_rot = Mat3::from_rotation_x(angles.1);
            let a2_rot = Mat3::from_rotation_x(angles.2);
            let a3_rot = Mat3::from_rotation_x(angles.3);

            // Create vectors in default position (Pointing along X-Axis) except base
            let a_b = self.cons.a_b.v;
            let a_1 = Vec3::new(0.0, self.cons.l_a1,  0.0);
            let a_2 = Vec3::new(0.0, self.cons.l_a2, 0.0);
            let a_3 = self.a_dec().v;

            // Multiply up
            Vectors( 
                base_rot * a_b,
                base_rot * a1_rot * a_1,
                base_rot * a1_rot * a2_rot * a_2,
                base_rot * a1_rot * a2_rot * a3_rot * a_3
            )
        }

        /// Get the the angles of the robot when moving to the given point with a fixed decoration axis
        pub fn get_with_fixed_dec(&self, point : Vec3, dec_angle : f32) -> Phis {
            // Rotate onto Y-Z plane
            let phi_b = top_down_angle(point) - PI/2.0;
            let rot_point = Mat3::from_rotation_z(-phi_b) * point;

            // Calculate the decoration vector
            let dec = self.a_dec().into_y();
            let dec_rot = Mat3::from_rotation_x(dec_angle) * dec.v;

            // Triganlge point
            let d_point = rot_point - dec_rot - self.cons.a_b.v;
            
            let phi_h1 = law_of_cosines(d_point.length(), self.cons.l_a1, self.cons.l_a2);      // Helper angle for phi_1 calc
            let gamma_2_ = law_of_cosines(self.cons.l_a2, self.cons.l_a1, d_point.length());    // Gamma 2 with side angles
            let mut phi_h = Vec3::Y.angle_between(d_point);                                             // Direct angle towards point

            if 0.0 > d_point.z {
                phi_h = -phi_h;
            }

            let phi_1 = phi_h + phi_h1;
            let phi_2 = gamma_2_ - PI;
            let phi_3 = dec_angle - (phi_1 + phi_2);

            Phis( phi_b, phi_1, phi_2, phi_3 )         
        }

        pub fn get_with_fixed_dec_s(&self, x : Option<f32>, y : Option<f32>, z : Option<f32>, dec_angle_o : Option<f32>) -> SyArmResult<Phis> {
            let point = Vec3::new(
                x.unwrap_or(self.vars.point.x),
                y.unwrap_or(self.vars.point.y),
                z.unwrap_or(self.vars.point.z)
            );

            let dec_angle = dec_angle_o.unwrap_or(self.vars.dec_angle);
            
            let phis = self.get_with_fixed_dec(point, dec_angle);
            let gammas = self.gammas_for_phis(&phis);
        
            if self.valid_gammas(gammas) { 
                Ok(phis)
            } else {
                let valids = self.valid_gammas_det(gammas);

                Err(SyArmError::new(format!(
                    "Point {} is out of range! (Gammas: {}, Dec: {}) (Valids: ({}, {}, {}, {}))", 
                        point, gammas, dec_angle, valids.0, valids.1, valids.2, valids.3).as_str(), ErrType::OutOfRange))
            }
        }

        pub fn stepper_axes(&self, base_angle : f32) -> Axes {
            let rot_x = Mat3::from_rotation_z(base_angle) * Vec3::NEG_X;
            Axes(
                Vec3::Z,
                rot_x, 
                rot_x,
                rot_x
            )
        } 
    //

    // Advanced velocity calculation
        pub fn actor_vectors(&self, vecs : &Vectors, phis : &Phis) -> Actors {
            let Vectors( a_b, a_1, a_2, a_3 ) = *vecs;
            let Axes( x_b, x_1, x_2, x_3 ) = self.stepper_axes(phis.0);

            let a_23 = a_2 + a_3;
            let a_123 = a_1 + a_23;
            let a_b123 = a_b + a_123;

            Actors(
                ( a_b123 ).cross( x_b ),
                ( a_123 ).cross( x_1 ),
                ( a_23 ).cross( x_2 ),
                ( a_3 ).cross( x_3 )
            )
        }

        pub fn accel_dyn(&self, phis : &Phis, omegas : Vec3) -> Vec3 {
            let Actors( eta_b, eta_1, eta_2, _ ) = self.actor_vectors(&vecs, phis);
            Vec3::new(
                eta_b * self.ctrl_base.accel_dyn(omegas.x),
                eta_1 * self.ctrl_a1.accel_dyn(omegas.y),
                eta_2 * self.ctrl_a2.accel_dyn(omegas.z)
            )
        }

        pub fn create_velocity(&self, vel : Vec3, phis : &Phis) -> Vec3 {
            let vecs = self.get_vectors_by_phis(phis);
            let Actors( eta_b, eta_1, eta_2, _ ) = self.actor_vectors(&vecs, phis);
            let Vectors( a_b, a_1, a_2, a_3 ) = vecs;

            let a_23 = a_2 + a_3;
            let a_123 = a_1 + a_23;
            let a_b123 = a_b + a_123;

            let eta_m = Mat3 {
                x_axis: eta_b,
                y_axis: eta_1,
                z_axis: eta_2
            };
            
            let vel_red = (a_b123.cross(vel) * a_b123.length().powi(-2)).cross(a_b + a_1 + a_2);
            
            eta_m.inverse() * vel_red
        }
    // 

    // Load / Inertia calculation
        pub fn get_cylinder_vecs(&self, vecs : &Vectors) -> CylVectors {
            let Vectors( _, a_1, a_2, _ ) = *vecs;

            let base_helper = Mat3::from_rotation_z(-self.ctrl_base.get_dist()) * Vec3::new(0.0, -self.cons.l_c1a, 0.0);

            CylVectors(
                (a_1 / 2.0 - base_helper, base_helper),
                (a_1 / 2.0 + a_2 / 2.0, a_1 / 2.0),
                (a_1 / 2.0 + a_2 / 2.0, a_2 / 2.0)
            )
        }

        pub fn get_inertias(&self, vecs : &Vectors) -> Inertias {
            let Vectors( a_b, a_1, a_2, a_3 ) = *vecs;
            let CylVectors( (c1_dir, c1_pos), (_, _), (c2_dir, c2_pos) ) = self.get_cylinder_vecs(vecs);

            let mut segments = vec![ (self.cons.m_a3, a_3) ];
            let j_3 = inertia_rod_constr(&segments) + inertia_point(a_3 + self.get_tool().unwrap().get_vec(), self.get_tool().unwrap().get_mass());

            segments.insert(0, (self.cons.m_a2, a_2));
            let j_2 = inertia_rod_constr(&segments) + inertia_point(a_2 + a_3 + self.get_tool().unwrap().get_vec(), self.get_tool().unwrap().get_mass());

            segments.insert(0, (self.cons.m_a1, a_1));
            let j_1 = inertia_rod_constr(&segments) + inertia_point(a_1 + a_2 + a_3 + self.get_tool().unwrap().get_vec(), self.get_tool().unwrap().get_mass());

            segments.insert(0, (self.cons.m_b, a_b));
            let j_b = inertia_rod_constr(&segments) + inertia_point(a_b + a_1 + a_2 + a_3 + self.get_tool().unwrap().get_vec(), self.get_tool().unwrap().get_mass());

            Inertias( 
                j_b.z_axis.length() / 1_000_000.0, 
                inertia_to_mass(j_1, c1_pos, c1_dir), 
                inertia_to_mass(j_2, c2_pos, c2_dir),
                (Mat3::from_rotation_z(-self.ctrl_base.get_dist()) * j_3).x_axis.length() / 1_000_000.0
            )
        }

        pub fn apply_inertias(&mut self, inertias : Inertias) {
            let Inertias( j_b, m_1, m_2, j_3 ) = inertias;

            self.ctrl_base.apply_load_inertia(j_b);
            self.ctrl_a1.cylinder.apply_load_inertia(m_1);
            self.ctrl_a2.cylinder.apply_load_inertia(m_2);
            self.ctrl_a3.apply_load_inertia(j_3);
        }

        pub fn get_forces(&self, vecs : &Vectors) -> Forces {
            let Vectors( _, a_1, a_2, a_3 ) = *vecs;
            let CylVectors( (c1_dir, c1_pos), (_, c2_pos_1), (c2_dir_2, c2_pos_2) ) = self.get_cylinder_vecs(vecs);

            let fg_load = G * self.vars.load;
            let fg_tool = G * self.get_tool().unwrap().get_mass();

            let fg_3 = G * self.cons.m_a3;
            let fg_2 = G * self.cons.m_a2;
            let fg_1 = G * self.cons.m_a1;

            let a_load = self.get_tool().unwrap().get_vec() + a_3;

            let (t_3, f_3) = forces_joint(&vec![ (fg_load + fg_tool, a_load), (fg_3, a_3 / 2.0) ], Vec3::ZERO);
            let (f_c2, f_2) = forces_segment(&vec![ (f_3, a_2), (fg_2, a_2 / 2.0) ], t_3, c2_pos_2, c2_dir_2);
            let (f_c1, _ ) = forces_segment(&vec![ (f_2, a_1), (f_c2, c2_pos_1), (fg_1, a_1 / 2.0) ], Vec3::ZERO, c1_pos, c1_dir);

            Forces( 0.0, f_c1.length(), f_c2.length(), t_3.length() / 1_000.0 )
        }

        pub fn apply_forces(&mut self, forces : Forces) {
            let Forces( t_b, f_1, f_2, t_3 ) = forces;

            self.ctrl_base.apply_load_force(t_b);
            self.ctrl_a1.cylinder.apply_load_force(f_1);
            self.ctrl_a2.cylinder.apply_load_force(f_2);
            self.ctrl_a3.apply_load_force(t_3);
        }
    // 

    // Update
        pub fn update_sim(&mut self) -> Vectors {
            let phis = self.get_all_phis();
            let vectors = self.get_vectors_by_phis(&phis);
            let points = self.get_points_by_phis(&phis);
            
            self.apply_forces(self.get_forces(&vectors));
            self.apply_inertias(self.get_inertias(&vectors));

            self.vars.point = points.3;

            vectors
        }
    //  

    // Control
        /// Moves the base by a relative angle \
        /// Angle in radians
        pub fn drive_base_rel(&mut self, angle : f32) {
            self.ctrl_base.drive(angle, self.cons.omega_b);
        }

        /// Moves the base to an absolute position \
        /// Angle in radians
        pub fn drive_base_abs(&mut self, angle : f32) {
            self.ctrl_base.drive_abs(angle, self.cons.omega_b);
        }

        pub fn drive_a1_rel(&mut self, angle : f32) {
            self.ctrl_a1.drive(angle, self.cons.c1_v);
        }

        pub fn drive_a1_abs(&mut self, angle : f32) {
            self.ctrl_a1.drive_abs(angle, self.cons.c1_v);
        }

        pub fn drive_a2_rel(&mut self, angle : f32) {
            self.ctrl_a2.drive(angle, self.cons.c2_v);
        }

        pub fn drive_a2_abs(&mut self, angle : f32) {
            self.ctrl_a2.drive_abs(angle, self.cons.c2_v);
        }

        pub fn drive_a3_rel(&mut self, angle : f32) {
            self.ctrl_a3.drive(angle, self.cons.omega_3);
        }

        pub fn drive_a3_abs(&mut self, angle : f32) {
            self.ctrl_a3.drive_abs(angle, self.cons.omega_3);
        }

        pub fn drive_to_angles(&mut self, angles : Gammas) {
            let Gammas( g_b, g_1, g_2, g_3 ) = angles;
            
            self.drive_base_abs(g_b);
            self.drive_a1_abs(g_1);
            self.drive_a2_abs(g_2);
            self.drive_a3_abs(g_3);
        }

        pub fn measure(&mut self, accuracy : u64) -> Result<(), (bool, bool, bool, bool)> {
            // self.ctrl_base.measure(2*PI, self.cons.omega_b, false);
            let a_1 = self.ctrl_a1.cylinder.measure(-(self.cons.l_c1a + self.cons.l_c1b), self.cons.c1_v, self.cons.meas_a1, accuracy);
            let a_2 = self.ctrl_a2.cylinder.measure(-(self.cons.l_c2a + self.cons.l_c2b), self.cons.c2_v, self.cons.meas_a2, accuracy);
            let a_3 = self.ctrl_a3.measure(-2.0*PI, self.cons.omega_3,  self.cons.meas_a3, accuracy);

            if a_1 & a_2 & a_3 {
                self.update_sim();
                Ok(())
            } else {
                Err((true, a_1, a_2, a_3))
            }
        }
    //

    // Async control
        /// Moves the base by a relative angle \
        /// Angle in radians
        pub fn drive_base_rel_async(&mut self, angle : f32) {
            self.ctrl_base.drive_async(self.ctrl_base.get_dist() + angle, self.cons.omega_b);
        }

        /// Moves the base to an absolute position \
        /// Angle in radians
        pub fn drive_base_abs_async(&mut self, angle : f32) {
            self.ctrl_base.drive_abs_async(angle, self.cons.omega_b);
        }

        pub fn drive_a1_rel_async(&mut self, angle : f32) {
            self.ctrl_a1.drive_async(angle, self.cons.c1_v);
        }

        pub fn drive_a1_abs_async(&mut self, angle : f32) {
            self.ctrl_a1.drive_abs_async(angle, self.cons.c1_v);
        }

        pub fn drive_a2_rel_async(&mut self, angle : f32) {
            self.ctrl_a2.drive_async(angle, self.cons.c2_v);
        }

        pub fn drive_a2_abs_async(&mut self, angle : f32) {
            self.ctrl_a2.drive_abs_async( angle, self.cons.c2_v);
        }

        pub fn drive_a3_rel_async(&mut self, angle : f32) {
            self.ctrl_a3.drive_async(angle, self.cons.omega_3);
        }

        pub fn drive_a3_abs_async(&mut self, angle : f32) {
            self.ctrl_a3.drive_abs_async( angle, self.cons.omega_3);
        }
        
        pub fn drive_to_angles_async(&mut self, angles : Gammas) {
            let Gammas( g_b, g_1, g_2, g_3 ) = angles;
            
            self.drive_base_abs_async(g_b);
            self.drive_a1_abs_async(g_1);
            self.drive_a2_abs_async(g_2);
            self.drive_a3_abs_async(g_3);
        }

        pub fn measure_async(&mut self, accuracy : u64) {
            self.ctrl_base.measure_async(0.0, 0.0, accuracy);
            self.ctrl_a1.cylinder.measure_async(-(self.cons.l_c1a + self.cons.l_c1b), self.cons.c1_v, accuracy);
            self.ctrl_a2.cylinder.measure_async(-(self.cons.l_c2a + self.cons.l_c2b), self.cons.c2_v, accuracy);
            self.ctrl_a3.measure_async(-2.0*PI, self.cons.omega_3, accuracy);
        }

        pub fn await_inactive(&self) {
            self.ctrl_base.ctrl.comms.await_inactive();
            self.ctrl_a1.cylinder.ctrl.comms.await_inactive();
            self.ctrl_a2.cylinder.ctrl.comms.await_inactive();
            self.ctrl_a3.ctrl.comms.await_inactive();
        }

        pub fn set_endpoint(&mut self) {
            self.ctrl_base.ctrl.set_endpoint(self.cons.meas_b);
            self.ctrl_a1.cylinder.ctrl.set_endpoint(self.cons.meas_a1);
            self.ctrl_a2.cylinder.ctrl.set_endpoint(self.cons.meas_a2);
            self.ctrl_a3.ctrl.set_endpoint(self.cons.meas_a3);
        }
    // 

    // Debug
        pub fn debug_pins(&self) {
            self.ctrl_base.ctrl.debug_pins();
            self.ctrl_a1.cylinder.ctrl.debug_pins();
            self.ctrl_a2.cylinder.ctrl.debug_pins();
            self.ctrl_a3.ctrl.debug_pins();
        }
    // 
}
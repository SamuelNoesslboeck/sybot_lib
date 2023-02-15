/// Library types
pub use glam::Vec3;
pub use glam::Mat3;

// Renamed Types 
pub type SyArmResult<T> = Result<T, SyArmError>;

/// Exchange tuple type for directional and positional vectors of the robot 
#[derive(Clone, Copy, Debug)]
pub struct CylVectors(
    /// First cylinder for first segment \
    /// ( Direction, Position )
    pub (Vec3, Vec3),    
    /// Second cylinder for first segment \
    /// ( Direction, Position )
    pub (Vec3, Vec3),       
    /// Second cylinder for second segment \
    /// ( Direction, Position )
    pub (Vec3, Vec3)        
);

#[derive(Copy, Clone, Debug)]
pub enum ErrType {
    None,
    
    // Movements
    OutOfRange,
    BadPins,

    // Interpreter
    GCodeFuncNotFound
}

#[derive(Debug)]
pub struct SyArmError 
{
    pub msg : String,
    pub err_type : ErrType
}

impl std::fmt::Display for SyArmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[SyArm-Error {}] {}", self.err_type as u64, self.msg)
    }
}

impl std::error::Error for SyArmError {
    
}

impl SyArmError {
    pub fn new_simple(err_type : ErrType) -> Self {
        Self {
            err_type: err_type,
            msg: String::new()
        }
    }

    pub fn new(msg : &str, err_type : ErrType) -> Self {
        Self {
            err_type: err_type,
            msg: String::from(msg)
        }
    }
}
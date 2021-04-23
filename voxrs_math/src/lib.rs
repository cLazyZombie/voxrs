mod aabb;
mod angle;
mod dir;
mod frustum;
//mod matrix4;
mod plane;
//mod quat;
mod ray;
mod rect;
mod sphere;

pub use aabb::Aabb;
pub use angle::Angle;
pub use dir::Dir;
pub use frustum::Frustum;
pub use glam::{IVec2, IVec3, Mat3, Mat4, Quat, Vec2, Vec3, Vec4};
pub use plane::Plane;
pub use ray::Ray;
pub use ray::RayAabbResult;
pub use rect::Rect2;
pub use sphere::Sphere;

mod chunk;

pub use chunk::WorldBlockCounts;
pub use chunk::WorldChunkCounts;
pub use chunk::BLOCK_COUNT_IN_CHUNKSIDE;
pub use chunk::TOTAL_BLOCK_COUNTS_IN_CHUNK;

mod chunk_pos;
pub use chunk_pos::ChunkPos;

mod block_pos;
pub use block_pos::BlockPos;

mod matrix4;
pub use matrix4::{get_matrix, set_matrix};

#[cfg(test)]
mod vector3;

#[cfg(test)]
mod quat;

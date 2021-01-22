# voxrs

## todo
- asset
 - reference count asset 
 - delete unused asset ( ref cont == 0 )
 - test AssetManager::get("path".into())
 - async loading
 - sperated not yet built asset (for performance reason)
 - if asset is invalid or not exists, print log and return default asset.

- Chunk
 - ChunkRenderer (64x64x64 cubes)
 - implements copy on write

- AssetPath From<&AssetPath> optimize
#!/bin/sh

cat src/vec_u32/mod.rs | sed -e 's/vec_u32/vec_u8/g' -e 's/StorageBlock = u32/StorageBlock = u8/g' -e 's/STORAGE_BLOCK_SIZE: usize = 32/STORAGE_BLOCK_SIZE: usize = 8/g' > src/vec_u8/mod.rs
cat src/vec_u32/mod.rs | sed -e 's/vec_u32/vec_u16/g' -e 's/StorageBlock = u32/StorageBlock = u16/g' -e 's/STORAGE_BLOCK_SIZE: usize = 32/STORAGE_BLOCK_SIZE: usize = 16/g' > src/vec_u16/mod.rs
cat src/vec_u32/mod.rs | sed -e 's/vec_u32/vec_u64/g' -e 's/StorageBlock = u32/StorageBlock = u64/g' -e 's/STORAGE_BLOCK_SIZE: usize = 32/STORAGE_BLOCK_SIZE: usize = 64/g' -e 's,+ Self::ERROR_TOO_MANY_VARIANTS,//+ Self::ERROR_TOO_MANY_VARIANTS,g' -e 's,const ERROR_TOO_MANY_VARIANTS: usize = [^;]*;,/*&*/,g' > src/vec_u64/mod.rs
cat src/vec_u32/mod.rs | sed -e 's/vec_u32/vec_u128/g' -e 's/StorageBlock = u32/StorageBlock = u128/g' -e 's/STORAGE_BLOCK_SIZE: usize = 32/STORAGE_BLOCK_SIZE: usize = 128/g' -e 's,+ Self::ERROR_TOO_MANY_VARIANTS,//+ Self::ERROR_TOO_MANY_VARIANTS,g' -e 's,const ERROR_TOO_MANY_VARIANTS: usize = [^;]*;,/*&*/,g' > src/vec_u128/mod.rs

cat src/vec_u32/mod.rs | sed -e '2iuse smallvec::SmallVec;' -e 's/vec_u32/smallvec_u32/g' -e 's/StorageBlock = u32/StorageBlock = u32/g' -e 's/STORAGE_BLOCK_SIZE: usize = 32/STORAGE_BLOCK_SIZE: usize = 32/g' -e 's/Storage = Vec<StorageBlock>/Storage = SmallVec<[StorageBlock; 4]>/g' -e 's/self.storage.append(&mut other.storage);/self.storage.extend_from_slice(\&other.storage);\nother.clear();/g' > src/smallvec_u32/mod.rs

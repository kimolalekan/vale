use super::StorageKind;
use crate::config;
use bincode;
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DBCompressionType, IteratorMode, Options, WriteBatch, DB,
};
use std::sync::{Arc, RwLock};

pub struct Storage {
    db: Arc<RwLock<DB>>,
}

impl Storage {
    fn with_cf_handle<F, R>(&self, cf: &str, f: F) -> Result<R, String>
    where
        F: FnOnce(&DB, &ColumnFamily) -> Result<R, String>,
    {
        let db = self.db.read().map_err(|_| "DB lock error".to_string())?;
        let cf_handle = db
            .cf_handle(cf)
            .ok_or_else(|| format!("Column family {} not found", cf))?;
        f(&db, cf_handle)
    }

    pub fn init() -> Result<Storage, String> {
        let path = config::DB_PATH;
        let mut opts = Options::default();
        opts.set_compression_type(DBCompressionType::Snappy);
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_level_compaction_dynamic_level_bytes(true);

        let cfs = vec![
            ColumnFamilyDescriptor::new(StorageKind::Account.name(), Options::default()),
            ColumnFamilyDescriptor::new(StorageKind::Transaction.name(), Options::default()),
            ColumnFamilyDescriptor::new(StorageKind::Contract.name(), Options::default()),
            ColumnFamilyDescriptor::new(StorageKind::Chain.name(), Options::default()),
            ColumnFamilyDescriptor::new(StorageKind::Index.name(), Options::default()),
            ColumnFamilyDescriptor::new(StorageKind::Analytics.name(), Options::default()),
        ];

        let db = DB::open_cf_descriptors(&opts, path, cfs).map_err(|e| e.to_string())?;
        Ok(Storage {
            db: Arc::new(RwLock::new(db)),
        })
    }

    pub fn put(&self, cf: &str, key: &[u8], value: &[u8], check_exist: bool) -> Result<(), String> {
        self.with_cf_handle(cf, |db, cf_handle| {
            if check_exist {
                let data_exists = self.exists(cf, key)?;
                if data_exists {
                    return Err("Record exists! Mutating value is not allowed.".to_string());
                }
            }
            db.put_cf(cf_handle, key, value.to_vec())
                .map_err(|e| e.to_string())?;
            Ok(())
        })?;
        self.update_analytics(cf.as_bytes())
    }

    pub fn get(&self, cf: &str, key: &[u8]) -> Result<Vec<u8>, String> {
        self.with_cf_handle(cf, |db, cf_handle| {
            db.get_cf(cf_handle, key)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "Key not found".to_string())
                .map(|data| data.to_vec())
        })
    }

    pub fn exists(&self, cf: &str, key: &[u8]) -> Result<bool, String> {
        self.with_cf_handle(cf, |db, cf_handle| match db.get_cf(cf_handle, key) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(e.to_string()),
        })
    }

    pub fn batch_put(&self, cf: &str, batch: Vec<(&[u8], &[u8])>) -> Result<(), String> {
        self.with_cf_handle(cf, |db, cf_handle| {
            let mut write_batch = WriteBatch::default();
            for (key, value) in batch {
                write_batch.put_cf(cf_handle, key, value);
            }
            db.write(write_batch).map_err(|e| e.to_string())
        })
    }

    pub fn batch_get(
        &self,
        cf: &str,
        start: usize,
        limit: usize,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, String> {
        self.with_cf_handle(cf, |db, cf_handle| {
            let mut iter = db.iterator_cf(cf_handle, IteratorMode::Start);

            for _ in 0..start {
                if iter.next().is_none() {
                    break;
                }
            }

            let result: Vec<(Vec<u8>, Vec<u8>)> = iter
                .take(limit)
                .filter_map(|item| item.ok())
                .map(|(key, value)| (key.to_vec(), value.to_vec()))
                .collect();

            Ok(result)
        })
    }

    fn update_analytics(&self, key: &[u8]) -> Result<(), String> {
        let cf = StorageKind::Analytics.name();

        let analytics = self.with_cf_handle(cf, |db, cf_handle| {
            db.get_cf(cf_handle, key)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "Analytics doesn't exist".to_string())
                .map(|data| data.to_vec())
        });

        let value = match analytics {
            Ok(data) => {
                let mut parse_analytics: i64 =
                    bincode::deserialize(&data).map_err(|e| e.to_string())?;
                parse_analytics += 1;
                bincode::serialize(&parse_analytics).map_err(|e| e.to_string())?
            }
            Err(_) => {
                let value: i64 = 1;
                bincode::serialize(&value).map_err(|e| e.to_string())?
            }
        };

        self.with_cf_handle(cf, |db, cf_handle| {
            db.put_cf(cf_handle, key, value).map_err(|e| e.to_string())
        })
    }

    pub fn get_analytics(&self, key: &[u8]) -> Result<i64, String> {
        let cf = StorageKind::Analytics.name();

        self.with_cf_handle(cf, |db, cf_handle| {
            let data = db
                .get_cf(cf_handle, key)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "Analytics doesn't exist".to_string())?;
            let analytics: i64 = bincode::deserialize(&data).map_err(|e| e.to_string())?;
            Ok(analytics)
        })
    }
}

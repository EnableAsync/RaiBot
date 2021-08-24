use cassandra_cpp::*;

pub struct TelephoneDb {
    session: Session,
}

impl Default for TelephoneDb {
    fn default() -> Self {
        let mut cluster = Cluster::default();
        let end_point = std::env::var("CASSANDRA_POINT").unwrap_or("127.0.0.1".to_string());
        dbg!("end point: {}", &end_point);
        cluster.set_contact_points(&end_point).unwrap();
        cluster.set_load_balance_round_robin();
        let session = cluster.connect().unwrap();

        TelephoneDb {
            session,
        }
    }
}

impl TelephoneDb {
    pub fn get_telephone(&self, qq: &str) -> Option<String> {
        if "" == qq {
            return None
        }
        let mut query = stmt!("SELECT * FROM tencent.telephone where qq = ?");
        query.bind(0, qq).unwrap();
        let result = self.session.execute(&query).wait().unwrap();
        println!("{}", result);
        if result.row_count() == 0 {
            return None
        }
        Some(result.first_row().unwrap().get_column_by_name("telephone").unwrap().to_string())
    }
}

#[cfg(test)]
mod test {
    use crate::db::TelephoneDb;

    #[test]
    fn test_telephone_db() {
        dotenv::dotenv().ok();
        let db = TelephoneDb::default();
        let phone = db.get_telephone("123");
        assert_eq!(phone, Some(String::from("123456789")))
    }
}
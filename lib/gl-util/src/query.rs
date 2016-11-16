use gl;
use gl::types::*;
use context::Context;
use std::i64;
use std::time::Duration;

/// Represents a query into OpenGL state.
pub struct Query {
    prev_end_query: QueryObject,
    end_query: QueryObject,
    start_timestamp: Option<i64>,

    pub(crate) context: gl::Context,
}

impl Query {
    pub fn new(context: &Context) -> Query {
        let context = context.inner();
        let _guard = ::context::ContextGuard::new(&context);

        let mut prev_end_query = QueryObject::null();
        let mut end_query = QueryObject::null();

        unsafe {
            // Generate the query object. This operation shouldn't fail.
            gl::gen_queries(1, &mut prev_end_query);
            assert!(prev_end_query != QueryObject::null(), "Failed to generate query object");

            gl::gen_queries(1, &mut end_query);
            assert!(end_query != QueryObject::null(), "Failed to generate query object");
        }

        Query {
            prev_end_query: prev_end_query,
            end_query: end_query,
            start_timestamp: None,

            context: context,
        }
    }

    pub fn time<F>(&mut self, func: F)
        where F: FnOnce()
    {
        let _guard = ::context::ContextGuard::new(&self.context);

        unsafe {
            gl::query_counter(self.prev_end_query, QueryCounterTarget::Timestamp);
        }

        // Run timed code.
        func();

        unsafe {
            gl::query_counter(self.end_query, QueryCounterTarget::Timestamp);
        }
    }

    pub fn time_latency<F>(&mut self, func: F)
        where F: FnOnce()
    {
        let _guard = ::context::ContextGuard::new(&self.context);

        let mut start_timestamp = i64::MIN;
        unsafe {
            gl::query_counter(self.prev_end_query, QueryCounterTarget::Timestamp);
            gl::get_i64v(Integer64Name::Timestamp, &mut start_timestamp);
        }
        assert!(start_timestamp != i64::MIN, "Failed to get start time");
        self.start_timestamp = Some(start_timestamp);

        // Run timed code.
        func();

        unsafe {
            gl::query_counter(self.end_query, QueryCounterTarget::Timestamp);
        }
    }

    pub fn elapsed(&self) -> Duration {
        let _guard = ::context::ContextGuard::new(&self.context);

        let mut prev_end_timestamp = i64::MIN;
        let mut end_timestamp = i64::MIN;
        unsafe {
            gl::get_query_object_i64v(self.prev_end_query, QueryResultType::Result, &mut prev_end_timestamp);
            gl::get_query_object_i64v(self.end_query, QueryResultType::Result, &mut end_timestamp);
        }
        assert!(prev_end_timestamp != i64::MIN, "Failed to get previous end time");
        assert!(end_timestamp != i64::MIN, "Failed to get end time");

        let diff = end_timestamp - prev_end_timestamp;
        assert!(diff > 0, "Operations took negative time???");

        let secs = diff / 1_000_000_000;
        let nanos = diff % 1_000_000_000;

        Duration::new(secs as u64, nanos as u32)
    }

    pub fn latency(&self) -> Option<Duration> {
        let _guard = ::context::ContextGuard::new(&self.context);

        let start_timestamp = self.start_timestamp.expect("Query did not include latency, use `time_latency()` instead");
        let mut prev_end_timestamp = i64::MIN;
        unsafe {
            gl::get_query_object_i64v(self.prev_end_query, QueryResultType::Result, &mut prev_end_timestamp);
        }
        assert!(prev_end_timestamp != i64::MIN, "Failed to get start time");

        let diff = prev_end_timestamp - start_timestamp;
        if diff > 0 {
            let secs = diff / 1_000_000_000;
            let nanos = diff % 1_000_000_000;

            Some(Duration::new(secs as u64, nanos as u32))
        } else {
            None
        }
    }
}

impl Drop for Query {
    fn drop(&mut self) {
        let _guard = ::context::ContextGuard::new(&self.context);
        unsafe {
            gl::delete_queries(1, &mut self.prev_end_query);
            gl::delete_queries(1, &mut self.end_query);
        }
    }
}

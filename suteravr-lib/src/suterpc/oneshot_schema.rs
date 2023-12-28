use crate::suterpc_oneshot_schema;

pub(crate) enum OneshotVariants {
    GetVersion = 0,
}

suterpc_oneshot_schema! {
    variant: GetVersion,
    enum Request {
        ClockingServer,
    },
    struct Response {
        version: String,
    },
}

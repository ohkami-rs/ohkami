pub(super) enum OkStatus {
    OK                  = 200,
    Created             = 201,
    NoContent           = 204,
}

pub(super) enum ErrStatus {
    BadRequest          = 400,
    Unauthorized        = 401,
    Forbidden           = 403,
    NotFound            = 404,
    InternalServerError = 500,
    NotImplemented      = 501,
}

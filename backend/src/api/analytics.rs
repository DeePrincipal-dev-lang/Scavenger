// This module previously contained legacy analytics endpoints
// (/analytics/participant/{id}, /analytics/global, /analytics/metrics/{name})
// that were never registered in main.rs and had no active consumers.
//
// Removed as part of #906 — unused/deprecated endpoint cleanup.
//
// Analytics data is available via the existing contract stats endpoint:
//   GET /api/v1/contracts/stats

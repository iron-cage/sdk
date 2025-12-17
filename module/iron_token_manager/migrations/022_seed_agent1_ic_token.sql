-- Seed dev IC token hash for agent_1 (demo/dev only)
-- Hash corresponds to dev_ic_token inserted in migration 017
<<<<<<< HEAD
=======
-- Token: eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJhZ2VudF9pZCI6ImFnZW50XzEiLCJidWRnZXRfaWQiOiJidWRnZXRfdGVzdCIsImlhdCI6MTc2NTU0OTExNSwiaXNzIjoiaXJvbi1jb250cm9sLXBhbmVsIiwicGVybWlzc2lvbnMiOlsibGxtOmNhbGwiLCJhbmFseXRpY3M6d3JpdGUiXX0.9cJZnr4OP7pIp30ntTR9NRgfNhDykEPFW_Ew1PYn-zU
>>>>>>> f326cba9b63f81a68e9971089276fd64a0ba039f
-- SHA-256: 897b52e23fde48c0c98b1f5aa80b80292cf1d8301adc51e7475d36068d53733a

UPDATE agents
SET ic_token_hash = '897b52e23fde48c0c98b1f5aa80b80292cf1d8301adc51e7475d36068d53733a',
    ic_token_created_at = strftime('%s','now')
WHERE id = 1 AND ic_token_hash IS NULL;

CREATE TABLE IF NOT EXISTS _migration_022_completed (id INTEGER PRIMARY KEY);

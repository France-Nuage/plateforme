-- Envelope-encrypt secret_values in billing.pending_instance_params.
--
-- Adds the four columns needed by frn_crypto envelope encryption (nonce, wrapped
-- DEK, DEK nonce, key version, algorithm) and drops the old cleartext BYTEA
-- column so secrets are never stored unencrypted.
--
-- Risk: SAFE - new nullable columns on a small, short-lived table.

-- atlas:nolint DS103
ALTER TABLE billing.pending_instance_params
    DROP COLUMN secret_values;

ALTER TABLE billing.pending_instance_params
    ADD COLUMN secret_ciphertext       BYTEA,
    ADD COLUMN secret_nonce            BYTEA,
    ADD COLUMN secret_dek_ciphertext   BYTEA,
    ADD COLUMN secret_dek_nonce        BYTEA,
    ADD COLUMN secret_key_version      INT,
    ADD COLUMN secret_algorithm        VARCHAR(50);

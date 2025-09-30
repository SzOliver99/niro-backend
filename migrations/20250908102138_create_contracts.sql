CREATE TABLE IF NOT EXISTS customer_contracts (
	id SERIAL PRIMARY KEY,
	UUID UUID UNIQUE DEFAULT uuid_generate_v4 (),
	contract_number VARCHAR(254) UNIQUE NOT NULL,
	contract_type VARCHAR(64) NOT NULL,
	annual_fee INT NOT NULL,
    first_payment BOOLEAN NOT NULL DEFAULT FALSE,
	payment_frequency VARCHAR(32) NOT NULL,
	payment_method VARCHAR(32) NOT NULL,
	customer_id INT NOT NULL REFERENCES customers (id) ON DELETE CASCADE,
	user_id INT REFERENCES users (id) ON DELETE SET NULL,
	created_by VARCHAR(254) NOT NULL,
	handle_at TIMESTAMPTZ(0) NOT NULL DEFAULT NOW(),
	CONSTRAINT contract_type_check CHECK (
		contract_type IN (
			'BonusLifeProgram',
			'LifeProgram',
			'AllianzCareNow',
			'HealthProgram',
			'MyhomeHomeInsurance',
			'MfoHomeInsurance',
			'CorporatePropertyInsurance',
			'Kgfb',
			'Casco',
			'TravelInsurance',
			'CondominiumInsurance',
			'AgriculturalInsurance'
		)
	),
	CONSTRAINT payment_frequency_check CHECK (payment_frequency IN ('Monthly', 'Quarterly', 'Semiannual', 'Annual')),
	CONSTRAINT payment_method_check CHECK (payment_method IN ('CreditCard', 'Transfer', 'DirectDebit', 'Check'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_customer_contracts_uuid ON customer_contracts (UUID);
CREATE UNIQUE INDEX IF NOT EXISTS idx_customer_contracts_contract_number ON customer_contracts (contract_number);
CREATE INDEX IF NOT EXISTS idx_customer_contracts_customer_id ON customer_contracts (customer_id);
CREATE INDEX IF NOT EXISTS idx_customer_contracts_user_id ON customer_contracts (user_id);
CREATE TABLE IF NOT EXISTS customer_contracts(
    id SERIAL PRIMARY KEY,
    uuid UUID UNIQUE DEFAULT uuid_generate_v4(),
    contract_number VARCHAR(254) NOT NULL,
    contract_type VARCHAR(64) NOT NULL,
    annual_fee INT NOT NULL,
    payment_frequency VARCHAR(32) NOT NULL,
    payment_method VARCHAR(32) NOT NULL,

    customer_id INT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    user_id INT REFERENCES users(id) ON DELETE SET NULL,
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
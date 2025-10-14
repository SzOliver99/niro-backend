CREATE TABLE IF NOT EXISTS customer_intervention_tasks (
	id SERIAL PRIMARY KEY,
	UUID UUID UNIQUE DEFAULT uuid_generate_v4 (),
	contract_number VARCHAR(254) NOT NULL,
	product_name VARCHAR(254) NOT NULL,
	outstanding_days INT NOT NULL,
	balance INT NOT NULL,
	processing_deadline TIMESTAMP NOT NULL,
	comment TEXT,
	"status" VARCHAR(32) NOT NULL,
	customer_id INT NOT NULL REFERENCES customers (id) ON DELETE CASCADE,
	user_id INT REFERENCES users (id) ON DELETE SET NULL,
	created_by VARCHAR(254) NOT NULL,
	CONSTRAINT task_status_check CHECK ("status" IN ('Pending', 'PaymentPromise', 'Processed', 'Nonpayment', 'PendingDeletion'))
);
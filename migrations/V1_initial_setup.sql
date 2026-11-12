CREATE SEQUENCE IF NOT EXISTS investment_id_seq;
CREATE SEQUENCE IF NOT EXISTS chart_history_data_id_seq;
CREATE TABLE IF NOT EXISTS investment (
    id TEXT PRIMARY KEY DEFAULT nextval('investment_id_seq'),
    name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    quantity DOUBLE,
    buy_price DOUBLE,
    buy_date TEXT,
    currency TEXT,
    investment_type TEXT NOT NULL,
    color TEXT,
    notes TEXT
);
CREATE TABLE IF NOT EXISTS chart_history_data (
    id TEXT PRIMARY KEY DEFAULT nextval('chart_history_data_id_seq'),
    investment_id TEXT NOT NULL,
    timestamp NUMERIC,
    open DOUBLE,
    high DOUBLE,
    low DOUBLE,
    close DOUBLE,
    volume DOUBLE,
    turnover DOUBLE,
    UNIQUE (investment_id, timestamp)
);
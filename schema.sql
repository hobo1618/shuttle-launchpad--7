CREATE TABLE IF NOT EXISTS articles (
  id SERIAL PRIMARY KEY,
  title VARCHAR(255),
  content TEXT,
  published_date VARCHAR(255)
)

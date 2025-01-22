-- ua vehicles data for the 2020
-- https://data.gov.ua/dataset/06779371-308f-42d7-895e-5a39833375f0

CREATE EXTERNAL TABLE cars
STORED AS CSV
LOCATION 'tz_opendata_z01012020_po01012021.csv'
OPTIONS ('has_header' 'true', 'format.delimiter' ';');

-- next-cell

-- Top 10 most popular models

SELECT "BRAND", "MODEL", count(*) AS count
FROM cars
GROUP BY "BRAND", "MODEL" ORDER BY COUNT DESC
LIMIT 10;

-- next-cell

-- Store result as JSON

COPY (
	SELECT "BRAND", "MODEL", count(*) AS count
	FROM cars
	GROUP BY "BRAND", "MODEL" ORDER BY COUNT DESC
	LIMIT 10
) TO 'result.json';

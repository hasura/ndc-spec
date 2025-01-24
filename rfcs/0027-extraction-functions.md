# Extraction Functions for Grouping

## Problem

We want to be able to support `EXTRACT` (aka `DATE_PART`) in groupings. For example, in Postgres:

```sql
SELECT
    EXTRACT(MONTH FROM invoice_date) AS month,
    SUM(amount) AS total
FROM invoices
GROUP BY EXTRACT(MONTH FROM invoice_date)
```

We want to be able to generalize this later to other hierarchical dimensions, e.g.:

- Geographical: continent, country, state, region, city, zip code, street, street number
- Categorical: e.g. product category, product subcategory, product ID
- File systems: Drive, folder, subfolder, filename
- Organizational: Department, Team, Employee

## Solution

Add `extraction_functions` to each `ScalarType` in the schema response:

```rust
pub struct ScalarType {
    // ...
    pub extraction_functions: BTreeMap<ExtractionFunctionName, ExtractionFunctionDefinition>,
}

pub struct ExtractionFunctionDefinition {
    /// The result type, which must be a defined scalar types in the schema response.
    result_type: ScalarTypeName,
    /// The meaning of this extraction function
    r#type: ExtractionFunctionType,
}

pub enum ExtractionFunctionType {
    Nanosecond,
    Microsecond,
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
    DayOfWeek,
    DayOfYear,
    Custom,
}
```

Add `extraction` to `Dimension`:

```rust
pub enum Dimension {
    Column {
        ...
        /// The name of the extraction function to apply to the selected value, if any
        extraction: Option<ExtractionFunctionName>,
    },
}
```

If `extraction` is provided, the column value should be computed first, and then the extraction function applied to the value. Data should be grouped by unique extracted values.

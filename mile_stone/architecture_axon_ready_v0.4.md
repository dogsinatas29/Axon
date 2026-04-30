# SYNAPSE Age Calculator - AXON READY ARCHITECTURE (v0.4)

## 🎯 PURPOSE

This document defines a **fully executable, AXON-ready architecture**.
It is directly translatable into code without ambiguity.

------------------------------------------------------------------------

# 📦 PROJECT STRUCTURE

    project/
    │
    ├── main.py
    ├── database.py
    ├── calculator.py
    ├── validators.py
    │
    ├── input_year.py
    ├── input_month.py
    ├── input_day.py

------------------------------------------------------------------------

# 🔗 DEPENDENCY GRAPH

``` mermaid
graph TD
    Main[main.py] --> DB[database.py]
    Main --> Calc[calculator.py]
    Main --> IY[input_year.py]
    Main --> IM[input_month.py]
    Main --> ID[input_day.py]

    IY --> V[validators.py]
    IM --> V
    ID --> V
```

------------------------------------------------------------------------

# 🔄 EXECUTION FLOW (DETERMINISTIC)

    START
     → input(name)
     → database.get_user(name)

        → [EXISTS]
            → bypass input
            → calculator.calculate_age
        → [NOT EXISTS]
            → input_year → validate_year
            → input_month → validate_month
            → input_day → validate_day
            → calculator.calculate_age

     → database.save_user
     → OUTPUT
    END

------------------------------------------------------------------------

# 📘 MODULE SPECIFICATIONS

## 1. main.py

### Role

-   Single entry point
-   Orchestrates full flow

### Interface

``` python
def main() -> None
```

### Responsibilities

-   Input name
-   Handle DB bypass logic
-   Call input modules
-   Call calculator
-   Persist result

------------------------------------------------------------------------

## 2. database.py

### Role

-   Data persistence (SQLite)

### Interfaces

``` python
def get_user(name: str) -> dict | None
def save_user(data: dict) -> None
```

### Constraints

-   Must initialize DB
-   Must guarantee unique name

------------------------------------------------------------------------

## 3. calculator.py

### Role

-   Age calculation logic

### Interface

``` python
def calculate_age(year: int, month: int, day: int) -> dict
```

### Output

``` python
{
    "korean_age": int,
    "international_age": int
}
```

------------------------------------------------------------------------

## 4. validators.py

### Role

-   Central validation

### Interfaces

``` python
def validate_year(year: int) -> int
def validate_month(month: int) -> int
def validate_day(year: int, month: int, day: int) -> int
```

------------------------------------------------------------------------

## 5. INPUT LAYER

### input_year.py

``` python
def input_year() -> int
```

### input_month.py

``` python
def input_month() -> int
```

### input_day.py

``` python
def input_day(year: int, month: int) -> int
```

------------------------------------------------------------------------

# 🧪 VALIDATION RULES

-   Year ≤ current year
-   Month: 1\~12
-   Day: valid calendar date
-   All invalid input must loop

------------------------------------------------------------------------

# 📊 DATA MODEL

  field         type
  ------------- ---------------
  id            INTEGER
  name          TEXT (UNIQUE)
  birth_year    INTEGER
  birth_month   INTEGER
  birth_day     INTEGER
  created_at    TIMESTAMP

  : user_records

------------------------------------------------------------------------

# 🔒 AXON COMPATIBILITY RULES

-   Each module must map to FILE
-   All interfaces must be deterministic
-   No implicit dependency
-   No hidden state
-   main.py is single orchestrator

------------------------------------------------------------------------

# 🧪 SUCCESS CONDITIONS

System is valid only if:

-   Code can be generated directly
-   All dependencies resolve
-   Execution completes without error
-   Input → Output flow is deterministic

------------------------------------------------------------------------

# 🚀 RESULT

This architecture guarantees:

-   Direct code generation (AXON compatible)
-   Deterministic execution
-   Zero ambiguity
-   Full pipeline integration

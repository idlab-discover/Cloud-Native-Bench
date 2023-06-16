from typing import List
from sqlalchemy import ForeignKey
from sqlalchemy import Text
from sqlalchemy import ARRAY
from sqlalchemy import Double
from sqlalchemy import DateTime
from sqlalchemy.sql import func
from sqlalchemy.orm import DeclarativeBase
from sqlalchemy.orm import Mapped
from sqlalchemy.orm import mapped_column
from sqlalchemy.orm import relationship


class Base(DeclarativeBase):
    pass


class BenchmarkResults(Base):
    __tablename__ = "benchmark_results"

    id: Mapped[int] = mapped_column(primary_key=True)

    name: Mapped[str] = mapped_column(Text())
    description: Mapped[str] = mapped_column(Text())
    data: Mapped[List["BenchmarkData"]] = relationship(
        cascade="all, delete-orphan")
    raw_data: Mapped[str] = mapped_column(Text())
    timestamp: Mapped[DateTime] = mapped_column(
        DateTime(timezone=True), server_default=func.now())
    generated_jupyter: Mapped[str] = mapped_column(Text(), nullable=True)

    def __repr__(self) -> str:
        return f"BenchmarkResults(id={self.id}, name={self.name}, description={self.description}, data={self.data}, raw_data={self.raw_data}, timestamp={self.timestamp}, generated_jupyter={self.generated_jupyter})"


class BenchmarkData(Base):
    __tablename__ = "benchmark_data"

    id: Mapped[int] = mapped_column(primary_key=True)
    benchmark_results_id: Mapped[int] = mapped_column(
        ForeignKey("benchmark_results.id"))

    parameter: Mapped[str] = mapped_column(Text())
    data_unit: Mapped[str] = mapped_column(Text())
    measurements: Mapped[List[int]] = mapped_column(ARRAY(Double()))
    measurement_name: Mapped[str] = mapped_column(Text())

    def __repr__(self) -> str:
        return f"BenchmarkData(id={self.id}, parameter={self.parameter}, data_unit={self.data_unit}, measurements={self.measurements})"

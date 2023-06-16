from sqlalchemy import create_engine
from sqlalchemy_utils import database_exists, create_database
from sqlalchemy import select
from sqlalchemy import update
from sqlalchemy import null
from sqlalchemy.orm import Session
from orm import Base
from orm import BenchmarkResults
from time import sleep
import nbformat
from textwrap import dedent
from dotenv import load_dotenv
import os


def main():
    load_dotenv()

    db_url = os.getenv("DATABASE_URL")

    if db_url is None:
        exit(1)

    engine = create_engine(db_url)

    if not database_exists(engine.url):
        create_database(engine.url)

    Base.metadata.create_all(engine)

    print("Analysis runner started.", flush=True)

    while True:
        sleep(10)

        stmt = select(BenchmarkResults).where(
            BenchmarkResults.generated_jupyter == null())

        with Session(engine) as session:
            # Loop over all the Benchmarks without jupyter notebook
            for row in session.scalars(stmt):
                # print(row)

                # Initialize the notebook
                notebook = init_notebook(row)

                # Loop over all the conducted experiments in the Benchmark
                for benchmark_iteration in row.data:
                    # Create cells with info about the results and plots

                    benchmark_info_cell = nbformat.v4.new_markdown_cell(dedent(
                        f"""\
                        ### Benchmark: {benchmark_iteration.parameter}
                        
                        #### Unit: {benchmark_iteration.data_unit}
                        """
                    ))

                    notebook.cells.append(benchmark_info_cell)

                    data_cell = nbformat.v4.new_code_cell(dedent(
                        f"""\
                        data = {{'measurements': {benchmark_iteration.measurements}}}

                        df = pd.DataFrame(data=data)
                        df
                        """
                    ))

                    notebook.cells.append(data_cell)

                    plot_cell = nbformat.v4.new_code_cell(dedent(
                        f"""\
                        plt = sns.kdeplot(data=df, x="measurements", fill=True)
                        plt.set_xlabel("{benchmark_iteration.measurement_name}")
                        plt.axvline(x=df['measurements'].mean())
                        """
                    ))

                    notebook.cells.append(plot_cell)

                # Create the JSON output of the notebook
                notebook_json = nbformat.writes(notebook)

                # Save the notebook to the Benchmark row
                stmt = update(BenchmarkResults).where(
                    BenchmarkResults.id == row.id).values(generated_jupyter=notebook_json)
                session.execute(stmt)
                session.commit()


def init_notebook(row: BenchmarkResults):
    notebook = nbformat.v4.new_notebook()

    info_cell = nbformat.v4.new_markdown_cell(dedent(
        f"""\
        ## Benchmark analysis

        ##### Automated Benchmark framework by michiel.vankenhove@ugent.be

        #### Benchmark name:

        {row.name}

        ### Benchmark timestamp:

        {row.timestamp}

        #### Benchmark description:

        {row.description}
        """
    ))

    import_cell = nbformat.v4.new_code_cell(dedent(
        """\
        import seaborn as sns
        import pandas as pd\
        """
    ))

    raw_data_cell = nbformat.v4.new_code_cell(dedent(
        f"""\
        raw_data = '''{row.raw_data}'''\
        """
    ))

    notebook.cells.append(info_cell)
    notebook.cells.append(import_cell)
    notebook.cells.append(raw_data_cell)

    return notebook


if __name__ == "__main__":
    main()


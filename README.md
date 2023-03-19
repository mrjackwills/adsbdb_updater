<p align="center">
	<img src='./.github/logo.svg' width='125px'/>
	<h1 align="center">adsbdb updater</h1>
</p>

<p align="center">
	This application, is designed to be executed in the <a href="https://github.com/mrjackwills/adsbdb" target='_blank' rel='noopener noreferrer'>adsbdb</a> docker network, to update any flightroute data issues.
	<br><br>
	Built in <a href='https://www.rust-lang.org/' target='_blank' rel='noopener noreferrer'>Rust</a>,
	for <a href='https://www.docker.com/' target='_blank' rel='noopener noreferrer'>Docker</a>,
	using <a href='https://www.postgresql.org/' target='_blank' rel='noopener noreferrer'>PostgreSQL</a>
	& <a href='https://www.redis.io/' target='_blank' rel='noopener noreferrer'>Redis</a> 
</p>

<hr>

### Data issues

Any actual data issues should be submitted via the <a href="https://github.com/mrjackwills/adsbdb/issues/new/choose" target='_blank' rel='noopener noreferrer'>adsbdb issues page</a>

### Run

Requires the adsbdb docker network, an `input.csv` and `.env`

```bash
./run.sh
```

### Build

```bash
cargo build --release
```
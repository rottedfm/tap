// cli.rs
use clap::{Parser, Subcommand};
use std::path::PathBuf;

const BANNER: &str = r#"
    .....               ..                  ....      ..
 .H8888888h.  ~-.    :**888H: `: .xH""    +^""888h. ~"888h
 888888888888x  `>  X   `8888k XX888     8X.  ?8888X  8888f
X~     `?888888hx~ '8hx  48888 ?8888    '888x  8888X  8888~
'      x8.^"*88*"  '8888 '8888 `8888    '88888 8888X   "88x:
 `-:- X8888x        %888>'8888  8888     `8888 8888X  X88x.
      488888>         "8 '888"  8888       `*` 8888X '88888X
    .. `"88*         .-` X*"    8888      ~`...8888X  "88888
  x88888nX"      .     .xhx.    8888       x8888888X.   `%8"
 !"*8888888n..  :    .H88888h.~`8888.>    '%"*8888888h.   "
'    "*88888888*    .~  `%88!` '888*~     ~    888888888!`
        ^"***"`           `"     ""            X888^"""
                                               `88f
                                                88
                                                ""
"#;

#[derive(Parser)]
#[command(name = "tap")]
#[command(about = "File investigation and export tool for mountable drives")]
#[command(before_help = BANNER)]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Inspect a drive and catalog its contents
    Inspect {
        /// Drive or path to inspect (e.g, /dev/sda or /mnt/evidence)
        drive: String,
    },
    /// Export files from a drive organized by type
    Export {
        /// Drive or path to export from (e.g, /dev/sda or /mnt/evidence)
        drive: String,

        /// Output directory for organized files
        #[arg(short, long)]
        output_dir: PathBuf,
    },
    // TODO: Discover -- find eleigables and output what is most likely data not boot partitions
}

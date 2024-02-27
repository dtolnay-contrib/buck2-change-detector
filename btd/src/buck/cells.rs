/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Context as _;

use crate::buck::types::CellName;
use crate::buck::types::CellPath;
use crate::buck::types::CellRelativePath;
use crate::buck::types::ProjectRelativePath;

pub struct CellResolver {
    cells: HashMap<CellName, ProjectRelativePath>,
    /// Sorted by path length, so the longest is first
    paths: Vec<(CellName, ProjectRelativePath)>,
}

impl CellResolver {
    pub fn new(file: &Path) -> anyhow::Result<Self> {
        let data = fs::read_to_string(file)
            .with_context(|| format!("When reading `{}`", file.display()))?;
        Self::parse(&data)
    }

    pub fn parse(data: &str) -> anyhow::Result<Self> {
        let json: HashMap<String, String> = serde_json::from_str(data)?;

        // We need to find the shortest path, as that will be the prefix and we want project relative paths
        let prefix = json
            .values()
            .min_by_key(|x| x.len())
            .ok_or_else(|| anyhow::anyhow!("Empty JSON object for the cells"))?
            .to_owned();
        let mut cells = HashMap::with_capacity(json.len());
        for (k, v) in json.into_iter() {
            match v.strip_prefix(&prefix) {
                None => {
                    return Err(anyhow::anyhow!(
                        "Expected key `{k}` to start with `{prefix}`, but got `{v}`"
                    ));
                }
                Some(rest) => {
                    cells.insert(
                        CellName::new(&k),
                        ProjectRelativePath::new(rest.trim_start_matches('/')),
                    );
                }
            }
        }
        let mut paths = cells
            .iter()
            .map(|(k, v)| ((*k).clone(), (*v).clone()))
            .collect::<Vec<_>>();
        paths.sort_by_key(|x| -(x.1.as_str().len() as isize));

        Ok(Self { cells, paths })
    }

    pub fn resolve(&self, path: &CellPath) -> anyhow::Result<ProjectRelativePath> {
        match self.cells.get(&path.cell()) {
            Some(prefix) => Ok(prefix.join(path.path().as_str())),
            None => Err(anyhow::anyhow!("Unknown cell, {:?}", path)),
        }
    }

    pub fn unresolve(&self, path: &ProjectRelativePath) -> anyhow::Result<CellPath> {
        // because we know self.paths has the longest match first, we just find the first match
        for (cell, prefix) in &self.paths {
            if let Some(x) = path.as_str().strip_prefix(prefix.as_str()) {
                let x = x.strip_prefix('/').unwrap_or(x);
                return Ok(cell.join(&CellRelativePath::new(x)));
            }
        }
        Err(anyhow::anyhow!(
            "Path has no cell which is a prefix `{:?}`",
            path
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell() {
        let value = serde_json::json!(
            {
                "inner1": "/Users/ndmitchell/repo/inner1",
                "inner2": "/Users/ndmitchell/repo/inner1/inside/inner2",
                "root": "/Users/ndmitchell/repo",
                "prelude": "/Users/ndmitchell/repo/prelude"
              }
        );
        let cells = CellResolver::parse(&serde_json::to_string(&value).unwrap()).unwrap();

        fn testcase(cells: &CellResolver, cell_path: &str, project_relative_path: &str) {
            let cell_path = CellPath::new(cell_path);
            let project_relative_path = ProjectRelativePath::new(project_relative_path);
            assert_eq!(cells.resolve(&cell_path).unwrap(), project_relative_path);
            assert_eq!(cells.unresolve(&project_relative_path).unwrap(), cell_path);
        }

        testcase(&cells, "inner1//magic/file.txt", "inner1/magic/file.txt");
        testcase(
            &cells,
            "inner2//magic/file.txt",
            "inner1/inside/inner2/magic/file.txt",
        );
        testcase(&cells, "root//file.txt", "file.txt");

        assert!(cells.resolve(&CellPath::new("missing//foo.txt")).is_err());
    }
}
use std::collections::{HashMap, HashSet};

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
struct Team {
    name: String,
    country: String,
    group: u8,
}

impl Team {
    fn new(name: String, country: String, group: u8) -> Team {
        Team { name, country, group }
    }
}

#[derive(Clone, Debug)]
struct Match {
    home_team: Team,
    away_team: Team,
}

impl Match {
    fn new(home_team: Team, away_team: Team) -> Match {
        Match { home_team, away_team }
    }
}

fn constraint_different_country(team1: &Team, team2: &Team) -> bool {
    team1.country != team2.country
}

struct CSPMatches {
    teams: Vec<Team>,  
    domains: HashMap<Team, HashSet<Team>>, 
    constraints: Vec<fn(&Team, &Team) -> bool>,  
    scheduled_matches: Vec<Match>,  
    group_requirements: HashMap<Team, HashMap<u8, (u8, u8)>>, 
}

impl CSPMatches {
    fn new(teams: Vec<Team>) -> CSPMatches {
        let domains = CSPMatches::initialize_domains(&teams);
        let group_requirements = CSPMatches::initialize_group_requirements(&teams);
        let constraints: Vec<fn(&Team, &Team) -> bool> = vec![
            constraint_different_country, 
        ];
        CSPMatches {
            teams,
            domains,
            constraints,
            scheduled_matches: Vec::new(),
            group_requirements,
        }
    }

    fn initialize_domains(teams: &[Team]) -> HashMap<Team, HashSet<Team>> {
        let mut domains: HashMap<Team, HashSet<Team>> = HashMap::new();

        for team in teams {
            let mut possible_opponents = HashSet::new();
            for opponent in teams {
                if team != opponent {
                    possible_opponents.insert(opponent.clone());
                }
            }
            domains.insert(team.clone(), possible_opponents);
        }

        domains
    }

    fn initialize_group_requirements(teams: &[Team]) -> HashMap<Team, HashMap<u8, (u8, u8)>> {
        let mut requirements = HashMap::new();
        for team in teams {
            let mut group_map = HashMap::new();
            for group in 1..=4 {
                group_map.insert(group, (0, 0));
            }
            requirements.insert(team.clone(), group_map);
        }
        requirements
    }

    fn satisfies_constraints(&self, team1: &Team, team2: &Team) -> bool {
        for constraint in &self.constraints {
            if !(constraint)(team1, team2) {
                return false;
            }
        }

        true
    }

    fn update_group_tracking(&mut self, team1: &Team, team2: &Team, home: bool) {
        if let Some(reqs) = self.group_requirements.get_mut(team1) {
            let entry = reqs.get_mut(&team2.group).unwrap();
            if home {
                entry.0 += 1;
            } else {
                entry.1 += 1;
            }
        }

        if let Some(reqs) = self.group_requirements.get_mut(team2) {
            let entry = reqs.get_mut(&team1.group).unwrap();
            if home {
                entry.1 += 1; 
            } else {
                entry.0 += 1; 
            }
        }
    }

    fn schedule_matches(&mut self) {
        let team_list: Vec<_> = self.teams.clone();
        
        for team in &team_list {
            let domain_list: Vec<_> = self.domains.get(team).unwrap().clone().into_iter().collect();
            
            for opponent in domain_list {
                if self.satisfies_constraints(team, &opponent) {
                    let new_match = Match::new(team.clone(), opponent.clone());
                    self.scheduled_matches.push(new_match.clone());
                    self.update_group_tracking(team, &opponent, true);

                    let return_match = Match::new(opponent.clone(), team.clone());
                    self.scheduled_matches.push(return_match);
                    self.update_group_tracking(&opponent, team, false);
                    self.domains.get_mut(team).unwrap().remove(&opponent);
                    self.domains.get_mut(&opponent).unwrap().remove(team);

                    break;
                }
            }
        }
    }

    fn display_matches(&self) {
        for m in &self.scheduled_matches {
            println!(
                "Match: {} (Home) vs {} (Away)",
                m.home_team.name, m.away_team.name
            );
        }
    }

    fn save_matches(&self) {
        let mut matches = self.scheduled_matches.clone();
        matches.sort_by(|a, b| a.home_team.name.cmp(&b.home_team.name));
        let mut wtr = csv::Writer::from_path("src/data/Scheduled_Matches.csv").unwrap();
        wtr.write_record(&["Home Team", "Away Team"]).unwrap();
        for m in matches {
            wtr.write_record(&[&m.home_team.name, &m.away_team.name]).unwrap();
        }
        wtr.flush().unwrap();
    }
}

fn main() {
    let teams = read_teams("src/data/Teams_Data.csv");
    let mut csp = CSPMatches::new(teams);

    csp.schedule_matches();
    csp.display_matches();
    csp.save_matches();
}

fn read_teams(file_path: &str) -> Vec<Team> {
    let mut teams = Vec::new();
    let file = std::fs::File::open(file_path).unwrap();
    let mut rdr = csv::Reader::from_reader(file);

    for result in rdr.records() {
        let record = result.unwrap();
        if record.get(0).map(|s| s == "team").unwrap_or(false) {
            continue;
        }

        let team_name = record.get(0).unwrap_or("").to_string();
        let country = record.get(1).unwrap_or("").to_string();
        let group: u8 = record.get(2).unwrap_or("0").parse().unwrap();

        let team = Team::new(team_name, country, group);
        teams.push(team);
    }

    teams
}

use std::fmt;

use std::collections::HashMap;
use std::collections::HashSet;

use multimap::MultiMap;

#[derive(Clone, Debug, PartialEq)]
pub enum TermType {
    Fall,
    Winter,
    Spring,
    Summer,
}

pub struct Course {
    pub name: String,
    pub credits: u8,
    pub availability: [bool; 4],
}

impl Course {
    pub fn new(name: String, credits: u8) -> Course {
        Course {
            name: name,
            credits: credits,
            availability: [false; 4],
        }
    }

    pub fn available_by<'a>(&'a mut self, term: &TermType) -> &'a mut Course {
        let index: usize = term.clone() as usize;

        self.availability[index] = true;

        self
    }

    pub fn not_available_by<'a>(&'a mut self, term: &TermType) -> &'a mut Course {
        let index: usize = term.clone() as usize;

        self.availability[index] = false;

        self
    }

    pub fn is_available(&self, term: &TermType) -> bool {
        let is_all_available = self.availability.iter().all(|&x| !x);

        if is_all_available {
            return is_all_available;
        }

        let index: usize = term.clone() as usize;

        self.availability[index]
    }
}

#[derive(Debug, PartialEq)]
pub struct Term {
    term_type: TermType,
    courses: HashSet<(String, u8)>,
    units: u8,
    unit_limit: u8,
}

impl Term {
    pub fn new(term: &TermType, unit_limit: u8) -> Term {
        Term {
            term_type: term.clone(),
            courses: HashSet::new(),
            units: 0,
            unit_limit: unit_limit,
        }
    }

    pub fn is_full(&self) -> bool {
        self.units == self.unit_limit
    }

    pub fn can_add_course(&self, course: &Course) -> bool {
        self.can_add_course_units(course.credits)
    }

    pub fn can_add_course_units(&self, units: u8) -> bool {
        units + self.units <= self.unit_limit
    }

    pub fn add(&mut self, course: &Course) {
        let is_added = self.courses.insert((course.name.clone(), course.credits));

        if is_added {
            self.units += course.credits;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.courses.len() == 0
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let term_name: &str = match self.term_type {
            TermType::Fall => "Fall",
            TermType::Winter => "Winter",
            TermType::Spring => "Spring",
            TermType::Summer => "Summer",
        };

        let term_header: String = format!("{}: {} units total\n", term_name, self.units);
        let mut term_body: String = String::new();

        for course in &self.courses {
            term_body.push_str(&format!("{}: {} units\n", course.0.clone(), course.1));
        }

        write!(f, "{}{}", term_header, term_body)
    }
}

pub struct Courses {
    master_list: HashMap<String, Course>,
    //VV TODO: Make copy of prereqs for processing VV
    prerequisites: MultiMap<String, String>,
    concurrencies: MultiMap<String, String>,
}

impl Courses {
    pub fn new() -> Courses {
        Courses {
            master_list: HashMap::new(),
            prerequisites: MultiMap::new(),
            concurrencies: MultiMap::new(),
        }
    }

    pub fn add_course(&mut self, course: Course) {
        self.master_list.insert(course.name.clone(), course);
    }

    pub fn remove_course(&mut self, course_name: &String) -> Option<Course> {
        self.master_list.remove(course_name)
    }

    fn add_prerequisite_to_concurrent(
        &mut self,
        course: &String,
        depends_on: &String,
        seen_courses: &mut HashSet<String>,
    ) {
        if seen_courses.contains(course) {
            return;
        }

        self.prerequisites
            .insert(course.clone(), depends_on.clone());
        seen_courses.insert(course.clone());

        if let Some(concurrents) = self.get_concurrents_for(course) {
            for concur_course in concurrents.0 {
                self.add_prerequisite_to_concurrent(&concur_course, depends_on, seen_courses);
            }
        }
    }

    pub fn add_prerequisite(&mut self, course: &String, depends_on: &String) {
        if self.concurrencies.contains_key(course) {
            self.add_prerequisite_to_concurrent(course, depends_on, &mut HashSet::new());
            return;
        }

        self.prerequisites
            .insert(course.clone(), depends_on.clone());
    }

    pub fn get_prerequisites(&self, course: &String) -> Option<HashSet<String>> {
        if let Some(found_prerequisites) = self.prerequisites.get_vec(&course.to_string()) {
            return Some(found_prerequisites.iter().cloned().collect());
        }

        None
    }

    pub fn remove_prerequisite(&mut self, course: &String, depends_on: &String) -> Option<String> {
        if let Some(c) = self.prerequisites.get_vec_mut(course) {
            if let Some(index) = c.iter().position(|x| x == depends_on) {
                return Some(c.remove(index));
            }
        }

        None
    }

    fn combine_concurrent_prerequisites(&mut self, course: &String, depends_on: &String) {
        let course_prerequisites = match self.get_prerequisites(course) {
            Some(x) => x,
            None => HashSet::new(),
        };

        let dependent_prerequisites = match self.get_prerequisites(depends_on) {
            Some(x) => x,
            None => HashSet::new(),
        };

        let mut prereqs_to_add: HashSet<String> = course_prerequisites
            .difference(&dependent_prerequisites)
            .cloned()
            .collect();
        for prereq_course in &prereqs_to_add {
            self.add_prerequisite(&depends_on, &prereq_course);
        }

        prereqs_to_add = dependent_prerequisites
            .difference(&course_prerequisites)
            .cloned()
            .collect();
        for prereq_course in &prereqs_to_add {
            self.add_prerequisite(&course, &prereq_course);
        }
    }

    pub fn add_concurrency(&mut self, course: &String, depends_on: &String) {
        if self.prerequisites.contains_key(course) || self.prerequisites.contains_key(depends_on) {
            self.combine_concurrent_prerequisites(course, depends_on);
        }

        self.concurrencies
            .insert(course.clone(), depends_on.clone());
        self.concurrencies
            .insert(depends_on.clone(), course.clone());
    }

    fn get_concurrents_with_memory(
        &self,
        course: &String,
        concurrents: &mut HashSet<String>,
    ) -> HashSet<String> {
        concurrents.insert(course.to_string());

        if let Some(found_concurs) = self.concurrencies.get_vec(&course.to_string()) {
            let mut found_set: HashSet<String> = found_concurs.iter().cloned().collect();

            for concur in found_concurs.iter() {
                if concurrents.contains(&concur.to_string()) {
                    continue;
                }

                found_set = found_set
                    .union(&self.get_concurrents_with_memory(concur, concurrents))
                    .cloned()
                    .collect();
            }

            return found_set;
        }

        HashSet::new()
    }

    fn get_concurrents_units(&self, concurrents: &HashSet<String>) -> u8 {
        concurrents
            .iter()
            .map(|x| {
                self.master_list
                    .get(x)
                    .expect("Course in concurrents not found in master list.")
            })
            .map(|x| x.credits)
            .sum()
    }

    pub fn get_concurrents_for(&self, course: &String) -> Option<(HashSet<String>, u8)> {
        let mut seen_courses: HashSet<String> = HashSet::new();

        let concurs_found = self.get_concurrents_with_memory(course, &mut seen_courses);

        if concurs_found.len() > 0 {
            let concurrent_units = self.get_concurrents_units(&concurs_found);

            return Some((concurs_found, concurrent_units));
        }

        None
    }

    pub fn remove_concurrency(
        &mut self,
        course: &String,
        depends_on: &String,
    ) -> Option<(String, String)> {
        if !self.concurrencies.contains_key(course) || !self.concurrencies.contains_key(depends_on)
        {
            return None;
        }

        let course_concurrents = self.concurrencies.get_vec_mut(course).unwrap();
        let dependent_index = course_concurrents
            .iter()
            .position(|x| x == depends_on)
            .unwrap();

        course_concurrents.remove(dependent_index);

        let dependent_concurrents = self.concurrencies.get_vec_mut(depends_on).unwrap();
        let course_index = dependent_concurrents
            .iter()
            .position(|x| x == course)
            .unwrap();

        dependent_concurrents.remove(course_index);

        Some((course.clone(), depends_on.clone()))
    }

    pub fn get_term_courses_for(&self, term: &TermType) -> Vec<String> {
        self.master_list
            .iter()
            .filter(|&x| x.1.is_available(term))
            .map(|x| x.1.name.clone())
            .collect()
    }

    fn get_next_term_for(&self, term: TermType) -> TermType {
        match term {
            TermType::Fall => TermType::Winter,
            TermType::Winter => TermType::Spring,
            TermType::Spring => TermType::Summer,
            TermType::Summer => TermType::Fall,
        }
    }

    pub fn get_terms(&self, term_unit_limits: [u8; 4]) -> Option<Vec<Term>> {
        let mut completed_terms: Vec<Term> = Vec::new();

        let mut fall_courses: Vec<String> = self.get_term_courses_for(&TermType::Fall);
        let mut winter_courses: Vec<String> = self.get_term_courses_for(&TermType::Winter);
        let mut spring_courses: Vec<String> = self.get_term_courses_for(&TermType::Spring);
        let mut summer_courses: Vec<String> = self.get_term_courses_for(&TermType::Summer);
        let mut prerequisites: MultiMap<String, String> = self.prerequisites.clone();

        let mut processed_term_courses: HashSet<String> = HashSet::new();
        let total_courses_count = self.len();

        let mut current_term = TermType::Fall;

        while processed_term_courses.len() < total_courses_count {
            let current_term_index: usize = current_term.clone() as usize;

            let mut term: Term = Term::new(&current_term, term_unit_limits[current_term_index]);
            let term_courses: &Vec<String> = match current_term {
                TermType::Fall => &fall_courses,
                TermType::Winter => &winter_courses,
                TermType::Spring => &spring_courses,
                TermType::Summer => &summer_courses,
            };

            for course_name in term_courses {
                if term.is_full() {
                    break;
                }

                let course: &Course = self.master_list.get(course_name).unwrap();

                if prerequisites.contains_key(course_name) || !term.can_add_course(course) {
                    continue;
                }

                if let Some(course_concurrents) = self.get_concurrents_for(&course_name) {
                    if !term.can_add_course_units(course_concurrents.1) {
                        continue;
                    }

                    for concur_course_name in course_concurrents.0 {
                        let concur_course: &Course =
                            self.master_list.get(&concur_course_name).unwrap();

                        term.add(&concur_course);
                        processed_term_courses.insert(concur_course.name.clone());
                    }
                } else {
                    term.add(&course);
                    processed_term_courses.insert(course.name.clone());
                }
            }

            fall_courses.retain(|x| !processed_term_courses.contains(x));
            winter_courses.retain(|x| !processed_term_courses.contains(x));
            spring_courses.retain(|x| !processed_term_courses.contains(x));
            summer_courses.retain(|x| !processed_term_courses.contains(x));

            for (.., value) in prerequisites.iter_all_mut() {
                value.retain(|x| !processed_term_courses.contains(x));
            }

            prerequisites.retain(|_k, v| v.len() > 0);

            if !term.is_empty() {
                completed_terms.push(term);
            }

            current_term = self.get_next_term_for(current_term);
        }

        if completed_terms.len() > 0 {
            return Some(completed_terms);
        }

        None
    }

    pub fn len(&self) -> usize {
        self.master_list.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_course_all_available() {
        let course_name: String = String::from("Test");
        let term = TermType::Fall;
        let mut my_course = Course::new(course_name, 4);

        assert!(my_course.is_available(&term));

        my_course
            .available_by(&TermType::Fall)
            .available_by(&TermType::Winter)
            .available_by(&TermType::Spring)
            .available_by(&TermType::Summer);

        assert!(my_course.is_available(&term));
    }

    #[test]
    fn test_course_one_not_available() {
        let course_name: String = String::from("Test");
        let term = TermType::Fall;

        let mut my_course = Course::new(course_name, 4);
        my_course.available_by(&term);

        assert!(!my_course.is_available(&TermType::Winter));
    }

    #[test]
    fn test_course_one_available() {
        let course_name: String = String::from("Test");
        let term = TermType::Fall;

        let mut my_course = Course::new(course_name, 4);
        my_course.available_by(&term);

        assert!(my_course.is_available(&term));
    }

    #[test]
    fn test_remove_nonexistant_concurrent() {
        let mut courses: Courses = Courses::new();

        assert_eq!(
            courses.remove_concurrency(&String::from("Test1"), &String::from("Test2")),
            None
        );
    }

    #[test]
    fn test_get_concurrents_none() {
        let courses: Courses = Courses::new();

        assert_eq!(courses.get_concurrents_for(&String::from("Test")), None);
    }

    #[test]
    fn test_get_concurrents() {
        let mut courses: Courses = Courses::new();
        let first_course: Course = Course::new(String::from("Test1"), 3);
        let first_course_name = first_course.name.to_string();

        let second_course: Course = Course::new(String::from("Test2"), 4);
        let second_course_name = second_course.name.to_string();

        let third_course: Course = Course::new(String::from("Test3"), 5);
        let third_course_name = third_course.name.to_string();

        courses.add_course(first_course);
        courses.add_course(second_course);
        courses.add_course(third_course);

        courses.add_concurrency(&first_course_name, &second_course_name);
        courses.add_concurrency(&second_course_name, &third_course_name);

        let test1_concurrents = courses.get_concurrents_for(&first_course_name);
        assert_ne!(test1_concurrents, None);
        assert_eq!(
            test1_concurrents,
            courses.get_concurrents_for(&second_course_name)
        );
        assert_eq!(
            test1_concurrents,
            courses.get_concurrents_for(&third_course_name)
        );

        assert_eq!(test1_concurrents.unwrap().1, 12);
    }

    #[test]
    fn test_get_term_no_prerequisites() {
        let mut courses: Courses = Courses::new();
        let first_course: Course = Course::new(String::from("CS 10"), 4);
        let second_course: Course = Course::new(String::from("CS 11"), 4);
        let third_course: Course = Course::new(String::from("CS 12"), 4);

        courses.add_course(first_course);
        courses.add_course(second_course);
        courses.add_course(third_course);

        let result: Option<Vec<Term>> = courses.get_terms([4, 4, 4, 4]);
        assert_ne!(result, None);

        let completed_terms: Vec<Term> = result.unwrap();

        for term in &completed_terms {
            println!("{}", term);
        }
        assert_eq!(completed_terms.len(), 3);
    }

    #[test]
    fn test_get_term_with_prerequisites_simple() {
        let mut courses: Courses = Courses::new();
        let first_course: Course = Course::new(String::from("CS 10"), 4);
        let first_course_name = first_course.name.to_string();

        let second_course: Course = Course::new(String::from("CS 11"), 4);
        let second_course_name = second_course.name.to_string();

        let third_course: Course = Course::new(String::from("CS 12"), 4);
        let third_course_name = third_course.name.to_string();

        courses.add_course(first_course);
        courses.add_course(second_course);
        courses.add_course(third_course);

        courses.add_prerequisite(&second_course_name, &first_course_name);
        courses.add_prerequisite(&third_course_name, &first_course_name);

        let result: Option<Vec<Term>> = courses.get_terms([8, 8, 8, 8]);
        assert_ne!(result, None);

        let completed_terms: Vec<Term> = result.unwrap();

        for term in &completed_terms {
            println!("{}", term);
        }
        assert_eq!(completed_terms.len(), 2);
    }

    #[test]
    fn test_concurrents_with_new_prerequisite() {
        let mut courses: Courses = Courses::new();
        let first_course: Course = Course::new(String::from("Test1"), 3);
        let first_course_name = first_course.name.to_string();

        let second_course: Course = Course::new(String::from("Test2"), 4);
        let second_course_name = second_course.name.to_string();

        let third_course: Course = Course::new(String::from("Test3"), 5);
        let third_course_name = third_course.name.to_string();

        let fourth_course: Course = Course::new(String::from("Test4"), 4);
        let fourth_course_name = fourth_course.name.to_string();

        courses.add_course(first_course);
        courses.add_course(second_course);
        courses.add_course(third_course);
        courses.add_course(fourth_course);

        courses.add_concurrency(&first_course_name, &second_course_name);
        courses.add_concurrency(&first_course_name, &third_course_name);

        let test1_concurrents_results = courses.get_concurrents_for(&first_course_name);
        assert_ne!(test1_concurrents_results, None);

        let test1_concurrents: (HashSet<String>, u8) = test1_concurrents_results.unwrap();
        assert_eq!(test1_concurrents.0.len(), 3);
        assert_eq!(courses.get_prerequisites(&first_course_name), None);

        courses.add_prerequisite(&first_course_name, &fourth_course_name);
        assert_ne!(courses.get_prerequisites(&first_course_name), None);
        assert_ne!(courses.get_prerequisites(&second_course_name), None);
        assert_ne!(courses.get_prerequisites(&third_course_name), None);
    }

    #[test]
    fn test_concurrents_with_existing_prerequisites() {
        let mut courses: Courses = Courses::new();
        let first_course: Course = Course::new(String::from("Test1"), 3);
        let first_course_name = first_course.name.to_string();

        let second_course: Course = Course::new(String::from("Test2"), 4);
        let second_course_name = second_course.name.to_string();

        let third_course: Course = Course::new(String::from("Test3"), 5);
        let third_course_name = third_course.name.to_string();

        let fourth_course: Course = Course::new(String::from("Test4"), 4);
        let fourth_course_name = fourth_course.name.to_string();

        let fifth_course: Course = Course::new(String::from("Test5"), 3);
        let fifth_course_name = fifth_course.name.to_string();

        let sixth_course: Course = Course::new(String::from("Test6"), 4);
        let sixth_course_name = sixth_course.name.to_string();

        courses.add_course(first_course);
        courses.add_course(second_course);
        courses.add_course(third_course);
        courses.add_course(fourth_course);

        courses.add_concurrency(&first_course_name, &second_course_name);
        courses.add_concurrency(&third_course_name, &fourth_course_name);
        assert_ne!(courses.get_concurrents_for(&first_course_name), None);
        assert_ne!(courses.get_concurrents_for(&third_course_name), None);

        courses.add_prerequisite(&first_course_name, &fifth_course_name);
        courses.add_prerequisite(&third_course_name, &sixth_course_name);
        assert_ne!(courses.get_prerequisites(&first_course_name), None);
        assert_ne!(courses.get_prerequisites(&third_course_name), None);

        courses.add_concurrency(&second_course_name, &fourth_course_name);

        let first_course_prerequisites_results: Option<HashSet<String>> =
            courses.get_prerequisites(&first_course_name);
        assert_ne!(first_course_prerequisites_results, None);

        let first_course_prerequisites: HashSet<String> =
            first_course_prerequisites_results.unwrap();
        assert_eq!(first_course_prerequisites.len(), 2);

        let second_course_prerequisite_results: Option<HashSet<String>> =
            courses.get_prerequisites(&second_course_name);
        assert_ne!(second_course_prerequisite_results, None);

        let second_course_prerequisites: HashSet<String> =
            second_course_prerequisite_results.unwrap();
        assert_eq!(second_course_prerequisites.len(), 2);

        let third_course_prerequisites_results: Option<HashSet<String>> =
            courses.get_prerequisites(&third_course_name);
        assert_ne!(third_course_prerequisites_results, None);

        let third_course_prerequisites: HashSet<String> =
            third_course_prerequisites_results.unwrap();
        assert_eq!(third_course_prerequisites.len(), 2);

        let fourth_course_prerequisites_results: Option<HashSet<String>> =
            courses.get_prerequisites(&fourth_course_name);
        assert_ne!(fourth_course_prerequisites_results, None);

        let fourth_course_prerequisites: HashSet<String> =
            fourth_course_prerequisites_results.unwrap();
        assert_eq!(fourth_course_prerequisites.len(), 2);
    }
}

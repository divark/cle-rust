use std::collections::HashMap;
use std::collections::HashSet;

use multimap::MultiMap;

pub enum Term {
	Fall,
	Winter,
	Spring,
	Summer,
}

pub struct Course {
	pub name: String,
	pub credits: u8,
	pub availability: [bool; 4]
}

impl Course {
	pub fn new(name: String, credits: u8) -> Course {
	    Course {
            name: name,
            credits: credits,
	        availability: [false; 4]
        }
    }

    pub fn available_by<'a>(&'a mut self, term: &Term) -> &'a mut Course {
        match &term {
            Term::Fall => self.availability[0] = true,
            Term::Winter => self.availability[1] = true,
            Term::Spring => self.availability[2] = true,
            Term::Summer => self.availability[3] = true,
        }
        self
    }

    pub fn not_available_by<'a>(&'a mut self, term: &Term) -> &'a mut Course {
        match &term {
            Term::Fall => self.availability[0] = false,
            Term::Winter => self.availability[1] = false,
            Term::Spring => self.availability[2] = false,
            Term::Summer => self.availability[3] = false,
        }
        self
    }

    pub fn is_available(&self, term: &Term) -> bool {
    	let is_all_available = self.availability.iter().all(|&x| x) || self.availability.iter().all(|&x| !x);

    	if is_all_available {
    		return is_all_available;
    	}

    	match &term {
            Term::Fall => self.availability[0],
            Term::Winter => self.availability[1],
            Term::Spring => self.availability[2],
            Term::Summer => self.availability[3],
        }
    }
}

pub struct Courses {
	pub master_list: HashMap<String, Course>,
	pub prerequisites: MultiMap<String, String>,
	pub concurrencies: MultiMap<String, String>
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

	pub fn remove_course(&mut self, course_name: String) -> Option<Course> {
		self.master_list.remove(&course_name)
	}

	pub fn add_prerequisite(&mut self, course: String, depends_on: String) {
		self.prerequisites.insert(course.clone(), depends_on.clone());
	}

	pub fn remove_prerequisite(&mut self, course: String, depends_on: String) -> Option<String> {
		if let Some(c) = self.prerequisites.get_vec_mut(&course) {
			if let Some(index) = c.iter().position(|x| *x == depends_on) {
				return Some(c.remove(index));
			}
		}

		None
	}

	pub fn add_concurrency(&mut self, course: &String, depends_on: &String) {
		self.concurrencies.insert(course.clone(), depends_on.clone());
        self.concurrencies.insert(depends_on.clone(), course.clone());
	}

    fn get_concurrents_with_memory(&self, course: &String, concurrents: &mut HashSet<String>) -> HashSet<String> {
        concurrents.insert(course.to_string());

        if let Some(found_concurs) = self.concurrencies.get_vec(&course.to_string()) {
            let mut found_set: HashSet<String> = found_concurs.iter().cloned().collect();

            for concur in found_concurs.iter() {
                if concurrents.contains(&concur.to_string()) {
                    continue;
                }

                found_set = found_set.union(&self.get_concurrents_with_memory(concur, concurrents)).cloned().collect();
            }

            return found_set;
        }

        HashSet::new()
    }

    pub fn get_concurrents_for(&self, course: &String) -> Option<HashSet<String>> {
        let mut seen_courses: HashSet<String> = HashSet::new();

        let concurs_found = self.get_concurrents_with_memory(course, &mut seen_courses);

        if concurs_found.len() > 0 {
            return Some(concurs_found);
        }

        None
    }

	pub fn remove_concurrency(&mut self, course: &String, depends_on: &String) -> bool {
        if !self.concurrencies.contains_key(course) || !self.concurrencies.contains_key(depends_on) {
            return false;
        }

        let course_concurrents = self.concurrencies.get_vec_mut(course).unwrap();
        let dependent_index = course_concurrents.iter().position(|x| x == depends_on).unwrap();

        course_concurrents.remove(dependent_index);

        let dependent_concurrents = self.concurrencies.get_vec_mut(depends_on).unwrap();
        let course_index = dependent_concurrents.iter().position(|x| x == course).unwrap();

        dependent_concurrents.remove(course_index);

        true
	}

	pub fn get_term_courses(&self, term: &Term) -> Vec<String> {
		self.master_list.iter()
			.filter(|&x| x.1.is_available(term))
			.filter(|&x| !self.prerequisites.contains_key(&x.1.name))
			.map(|x| x.1.name.clone())
			.collect()
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
		let term = Term::Fall;
		let mut my_course = Course::new(course_name, 4);

		assert!(my_course.is_available(&term));

		my_course.available_by(&Term::Fall)
    		.available_by(&Term::Winter)
    		.available_by(&Term::Spring)
    		.available_by(&Term::Summer);

		assert!(my_course.is_available(&term));
	}

	#[test]
	fn test_course_one_not_available() {
		let course_name: String = String::from("Test");
		let term = Term::Fall;

		let mut my_course = Course::new(course_name, 4);
		my_course.available_by(&term);

		assert!(!my_course.is_available(&Term::Winter));
	}

	#[test]
	fn test_course_one_available() {
		let course_name: String = String::from("Test");
		let term = Term::Fall;

		let mut my_course = Course::new(course_name, 4);
		my_course.available_by(&term);

		assert!(my_course.is_available(&term));
	}

	#[test]
	fn test_remove_nonexistant_concurrent() {
		let mut courses: Courses = Courses::new();

		assert_eq!(courses.remove_concurrency(&String::from("Test1"), &String::from("Test2")), false);
	}

    #[test]
    fn test_get_concurrents_none() {
        let courses: Courses = Courses::new();

        assert_eq!(courses.get_concurrents_for(&String::from("Test")), None);
    }

    #[test]
    fn test_get_concurrents_simple() {
        let mut courses: Courses = Courses::new();
        let first_course: String = String::from("Test1");
        let second_course: String = String::from("Test2");
        let third_course: String = String::from("Test3");

        courses.add_concurrency(&first_course, &second_course);
        courses.add_concurrency(&second_course, &third_course);

        let test1_concurrents = courses.get_concurrents_for(&first_course);
        assert_ne!(test1_concurrents, None);
        assert_eq!(test1_concurrents, courses.get_concurrents_for(&second_course));
        assert_eq!(test1_concurrents, courses.get_concurrents_for(&third_course));
    }
}

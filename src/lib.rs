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

	pub fn add_concurrency(&mut self, course: String, depends_on: String) {
		self.concurrencies.insert(course.clone(), depends_on.clone());
	}

	pub fn remove_concurrency(&mut self, course: String, depends_on: String) -> Option<String> {
		if let Some(c) = self.concurrencies.get_vec_mut(&course) {
			if let Some(index) = c.iter().position(|x| *x == depends_on) {
				return Some(c.remove(index));
			}
		}

		None
	}

    pub fn get_concurrents(&self, course: String) -> Vec<String> {

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
	fn test_no_concurrent() {
		let mut courses: Courses = Courses::new();

		assert_eq!(courses.remove_concurrency(String::from("Test1"), String::from("Test2")), None);
	}
}

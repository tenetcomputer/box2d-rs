use super::super::draw::*;
use super::super::settings::*;
use super::super::test::*;
use box2d_rs::b2_body::*;
use box2d_rs::b2_fixture::*;
use box2d_rs::b2_contact::G_BLOCK_SOLVE;
use box2d_rs::b2_math::*;
use box2d_rs::b2_common::*;
use box2d_rs::b2rs_common::UserDataType;
use box2d_rs::b2_world::*;
use box2d_rs::b2_world_callbacks::*;
use box2d_rs::shapes::b2_circle_shape::*;
use box2d_rs::shapes::b2_edge_shape::*;
use box2d_rs::shapes::b2_polygon_shape::*;

use glium::backend::Facade;
use std::cell::RefCell;
use std::rc::Rc;

use glium::glutin::event::{ElementState, KeyboardInput, VirtualKeyCode};

// use std::sync::atomic::Ordering;

pub(crate) struct BoxStack<D: UserDataType> {
	base: TestBasePtr<D>,
	destruction_listener: B2destructionListenerPtr<D>,
	contact_listener: B2contactListenerPtr<D>,
	m_bodies: Vec<BodyPtr<D>>,
	m_bullet: Option<BodyPtr<D>>,
}

impl<D: UserDataType> BoxStack<D> {
	const E_COLUMN_COUNT: usize = 1;
	const E_ROW_COUNT: usize = 15;

	pub fn new<F: Facade>(global_draw: TestBedDebugDrawPtr) -> TestPtr<D, F> {
		let base = Rc::new(RefCell::new(Test::new(global_draw.clone())));
		let result_ptr = Rc::new(RefCell::new(Self {
			base: base.clone(),
			destruction_listener: Rc::new(RefCell::new(B2testDestructionListenerDefault {
				base: Rc::downgrade(&base),
			})),
			contact_listener: Rc::new(RefCell::new(B2testContactListenerDefault {
				base: Rc::downgrade(&base),
			})),
			m_bodies: Vec::new(),
			m_bullet: None,
		}));

		{
			let mut self_ = result_ptr.borrow_mut();
			{
				let world = base.borrow().m_world.clone();
				let mut world = world.borrow_mut();
				world.set_destruction_listener(self_.destruction_listener.clone());
				world.set_contact_listener(self_.contact_listener.clone());
				world.set_debug_draw(global_draw);
			}
			self_.init();
		}

		return result_ptr;

	}

	fn init(&mut self) {
		let m_world = self.base.borrow().m_world.clone();
		{
			let bd = B2bodyDef::default();
			let ground = B2world::create_body(m_world.clone(), &bd);

			let mut shape = B2edgeShape::default();
			shape.set_two_sided(B2vec2::new(-40.0, 0.0), B2vec2::new(40.0, 0.0));
			B2body::create_fixture_by_shape(ground.clone(), Rc::new(RefCell::new(shape)), 0.0);

			shape.set_two_sided(B2vec2::new(20.0, 0.0), B2vec2::new(20.0, 20.0));
			B2body::create_fixture_by_shape(ground.clone(), Rc::new(RefCell::new(shape)), 0.0);
		}

		let xs = [0.0, -10.0, -5.0, 5.0, 10.0];

		{
			for j in 0..Self::E_COLUMN_COUNT {
				let mut shape = B2polygonShape::default();
				shape.set_as_box(0.5, 0.5);

				for i in 0..Self::E_ROW_COUNT {
					let mut fd = B2fixtureDef::default();
					fd.shape = Some(Rc::new(RefCell::new(shape)));
					fd.density = 1.0;
					fd.friction = 0.3;

					let mut bd = B2bodyDef::default();
					bd.body_type = B2bodyType::B2DynamicBody;
					bd.angle = B2_PI / 2.0;

					// i32 n = j * E_ROW_COUNT + i;
					// b2_assert(n < E_ROW_COUNT * E_COLUMN_COUNT);
					// m_indices[n] = n;
					// bd.user_data = m_indices + n;

					let x: f32 = 0.0;
					//f32 x = random_float(-0.02, 0.02);
					//f32 x = i % 2 == 0 ? -0.01 : 0.01;
					bd.position.set(xs[j] + x, 0.55 + 1.1 * i as f32);

					let body = B2world::create_body(m_world.clone(), &bd);
					self.m_bodies.push(body.clone());

					B2body::create_fixture(body.clone(), &fd);
				}
			}
		}
	}
}

impl<D: UserDataType, F: Facade> TestDyn<D, F> for BoxStack<D> {
	fn get_base(&self) -> TestBasePtr<D> {
		return self.base.clone();
	}
	fn keyboard(&mut self, key: &KeyboardInput) {
		if key.state != ElementState::Pressed {
			match key.virtual_keycode {
				Some(VirtualKeyCode::Comma) => {
					let m_world = self.base.borrow().m_world.clone();
					if let Some(bullet) = self.m_bullet.clone() {
						m_world.borrow_mut().destroy_body(bullet);
						self.m_bullet = None;
					}

					{
						let mut shape = B2circleShape::default();
						shape.base.m_radius = 0.25;

						let mut fd = B2fixtureDef::default();
						fd.shape = Some(Rc::new(RefCell::new(shape)));
						fd.density = 20.0;
						fd.restitution = 0.05;

						let mut bd = B2bodyDef::default();
						bd.body_type = B2bodyType::B2DynamicBody;
						bd.bullet = true;
						bd.position.set(-31.0, 5.0);

						let bullet = B2world::create_body(m_world.clone(), &bd);
						self.m_bullet = Some(bullet.clone());
						B2body::create_fixture(bullet.clone(), &fd);

						bullet
							.borrow_mut()
							.set_linear_velocity(B2vec2::new(400.0, 0.0));
					}
				}
				Some(VirtualKeyCode::B) => {
					unsafe {
					let g_block_solve_lower: bool = G_BLOCK_SOLVE;
					G_BLOCK_SOLVE = !g_block_solve_lower;
					}
				}
				_ => (),
			}
		}
	}
	fn step(
		&mut self,
		ui: &imgui::Ui<'_>,
		display: &F,
		target: &mut glium::Frame,
		settings: &mut Settings,
		camera: &mut Camera,
	) {
		Test::step(self.base.clone(), ui, display, target, settings, *camera);
		let mut g_block_solve_lower: bool = false;
		unsafe {
			g_block_solve_lower = G_BLOCK_SOLVE;
		}

		let mut base = self.base.borrow_mut();

			base.g_debug_draw.borrow().draw_string(
				ui,
				B2vec2::new(5.0, base.m_text_line as f32),
				"Press: (,) to launch a bullet.",
			);
			base.m_text_line += base.m_text_increment;
			base.g_debug_draw.borrow().draw_string(
				ui,
				B2vec2::new(5.0, base.m_text_line as f32),
				&format!("Blocksolve = {0}", g_block_solve_lower),
			);
			base.m_text_line += base.m_text_increment;

		//if (m_stepCount == 300)
		//{
		//	if (m_bullet != NULL)
		//	{
		//		m_world->DestroyBody(m_bullet);
		//		m_bullet = NULL;
		//	}

		//	{
		//		b2CircleShape shape;
		//		shape.m_radius = 0.25f;

		//		b2FixtureDef fd;
		//		fd.shape = &shape;
		//		fd.density = 20.0f;
		//		fd.restitution = 0.05f;

		//		b2BodyDef bd;
		//		bd.type = b2_dynamicBody;
		//		bd.bullet = true;
		//		bd.position.Set(-31.0f, 5.0f);

		//		m_bullet = m_world->CreateBody(&bd);
		//		m_bullet->CreateFixture(&fd);

		//		m_bullet->SetLinearVelocity(b2Vec2(400.0f, 0.0f));
		//	}
		//}
	}
}

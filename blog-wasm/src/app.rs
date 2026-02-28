use crate::api::ApiClient;
use crate::models::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

pub enum Msg {
    // Auth messages
    UpdateRegisterUsername(String),
    UpdateRegisterEmail(String),
    UpdateRegisterPassword(String),
    UpdateLoginUsername(String),
    UpdateLoginPassword(String),
    Register,
    Login,
    Logout,
    AuthSuccess(AuthResponse),

    // Post messages
    UpdatePostTitle(String),
    UpdatePostContent(String),
    LoadPosts,
    PostsLoaded(PostsResponse),
    CreatePost,
    PostCreated(Post),
    EditPost(i64),
    UpdatePost(i64, String, String),
    SavePost(i64),
    PostUpdated(Post),
    DeletePost(i64),
    PostDeleted(i64),
    CancelEdit,

    // UI messages
    Error(String),
}

#[derive(Clone, PartialEq)]
enum EditState {
    None,
    Editing {
        id: i64,
        title: String,
        content: String,
    },
}

pub struct App {
    // Auth state
    user: Option<User>,
    token: Option<String>,

    // Forms
    register_username: String,
    register_email: String,
    register_password: String,
    login_username: String,
    login_password: String,
    post_title: String,
    post_content: String,

    // Posts
    posts: Vec<Post>,
    posts_total: i64,

    // UI state
    loading: bool,
    error: Option<String>,
    edit_state: EditState,
    edit_form_data: Option<(i64, String, String)>,

    // API client
    api: ApiClient,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        // Check for existing token
        let token = ApiClient::get_token();
        let user = None;

        Self {
            user,
            token,
            register_username: String::new(),
            register_email: String::new(),
            register_password: String::new(),
            login_username: String::new(),
            login_password: String::new(),
            post_title: String::new(),
            post_content: String::new(),
            posts: Vec::new(),
            posts_total: 0,
            loading: false,
            error: None,
            edit_state: EditState::None,
            edit_form_data: None,
            api: ApiClient::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            // Auth form updates
            Msg::UpdateRegisterUsername(val) => {
                self.register_username = val;
                true
            }
            Msg::UpdateRegisterEmail(val) => {
                self.register_email = val;
                true
            }
            Msg::UpdateRegisterPassword(val) => {
                self.register_password = val;
                true
            }
            Msg::UpdateLoginUsername(val) => {
                self.login_username = val;
                true
            }
            Msg::UpdateLoginPassword(val) => {
                self.login_password = val;
                true
            }

            // Register
            Msg::Register => {
                if self.register_username.is_empty()
                    || self.register_email.is_empty()
                    || self.register_password.is_empty()
                {
                    self.error = Some("All fields are required".to_string());
                    return true;
                }

                self.loading = true;
                self.error = None;

                let req = RegisterRequest {
                    username: self.register_username.clone(),
                    email: self.register_email.clone(),
                    password: self.register_password.clone(),
                };

                let api = self.api.clone();
                let link = ctx.link().clone();

                spawn_local(async move {
                    match api.register(&req).await {
                        Ok(response) => {
                            ApiClient::save_token(&response.token);
                            link.send_message(Msg::AuthSuccess(response));
                        }
                        Err(e) => link.send_message(Msg::Error(e)),
                    }
                });

                false
            }

            // Login
            Msg::Login => {
                if self.login_username.is_empty() || self.login_password.is_empty() {
                    self.error = Some("Username and password are required".to_string());
                    return true;
                }

                self.loading = true;
                self.error = None;

                let req = LoginRequest {
                    username: self.login_username.clone(),
                    password: self.login_password.clone(),
                };

                let api = self.api.clone();
                let link = ctx.link().clone();

                spawn_local(async move {
                    match api.login(&req).await {
                        Ok(response) => {
                            ApiClient::save_token(&response.token);
                            link.send_message(Msg::AuthSuccess(response));
                        }
                        Err(e) => link.send_message(Msg::Error(e)),
                    }
                });

                false
            }

            // Logout
            Msg::Logout => {
                ApiClient::clear_token();
                self.token = None;
                self.user = None;
                self.login_username.clear();
                self.login_password.clear();
                true
            }

            // Auth success
            Msg::AuthSuccess(response) => {
                self.token = Some(response.token.clone());
                self.user = Some(response.user);
                self.loading = false;
                self.register_username.clear();
                self.register_email.clear();
                self.register_password.clear();
                self.login_username.clear();
                self.login_password.clear();

                // Load posts after successful auth
                ctx.link().send_message(Msg::LoadPosts);
                true
            }

            // Post form updates
            Msg::UpdatePostTitle(val) => {
                self.post_title = val;
                true
            }
            Msg::UpdatePostContent(val) => {
                self.post_content = val;
                true
            }

            // Load posts
            Msg::LoadPosts => {
                self.loading = true;

                let api = self.api.clone();
                let link = ctx.link().clone();

                spawn_local(async move {
                    match api.list_posts(10, 0).await {
                        Ok(response) => link.send_message(Msg::PostsLoaded(response)),
                        Err(e) => link.send_message(Msg::Error(e)),
                    }
                });

                false
            }

            Msg::PostsLoaded(response) => {
                self.posts = response.posts;
                self.posts_total = response.total;
                self.loading = false;
                true
            }

            // Create post
            Msg::CreatePost => {
                if self.post_title.is_empty() || self.post_content.is_empty() {
                    self.error = Some("Title and content are required".to_string());
                    return true;
                }

                self.loading = true;
                self.error = None;

                let req = CreatePostRequest {
                    title: self.post_title.clone(),
                    content: self.post_content.clone(),
                };

                let api = self.api.clone();
                let link = ctx.link().clone();

                spawn_local(async move {
                    match api.create_post(&req).await {
                        Ok(post) => link.send_message(Msg::PostCreated(post)),
                        Err(e) => link.send_message(Msg::Error(e)),
                    }
                });

                false
            }

            Msg::PostCreated(post) => {
                self.posts.insert(0, post);
                self.posts_total += 1;
                self.post_title.clear();
                self.post_content.clear();
                self.loading = false;
                true
            }

            // Edit post - начинаем редактирование
            Msg::EditPost(id) => {
                if let Some(post) = self.posts.iter().find(|p| p.id == id) {
                    // Сохраняем данные в локальное состояние
                    self.edit_form_data = Some((id, post.title.clone(), post.content.clone()));
                    self.edit_state = EditState::Editing {
                        id,
                        title: post.title.clone(),
                        content: post.content.clone(),
                    };
                }
                true
            }

            // Update post - только локальное обновление формы
            Msg::UpdatePost(id, title, content) => {
                // Обновляем локальное состояние, но не отправляем на сервер
                self.edit_form_data = Some((id, title.clone(), content.clone()));
                self.edit_state = EditState::Editing { id, title, content };
                true
            }

            // Save post - отправка на сервер
            Msg::SavePost(id) => {
                if let Some((_, title, content)) = self.edit_form_data.take() {
                    self.loading = true;
                    self.error = None;

                    let req = UpdatePostRequest {
                        title: Some(title.clone()),
                        content: Some(content.clone()),
                    };

                    let api = self.api.clone();
                    let link = ctx.link().clone();

                    spawn_local(async move {
                        match api.update_post(id, &req).await {
                            Ok(post) => link.send_message(Msg::PostUpdated(post)),
                            Err(e) => link.send_message(Msg::Error(e)),
                        }
                    });
                }
                true
            }

            // Post updated successfully
            Msg::PostUpdated(post) => {
                if let Some(index) = self.posts.iter().position(|p| p.id == post.id) {
                    self.posts[index] = post;
                }
                // Закрываем форму после успешного сохранения
                self.edit_state = EditState::None;
                self.edit_form_data = None;
                self.loading = false;
                true
            }

            // Delete post
            Msg::DeletePost(id) => {
                self.loading = true;

                let api = self.api.clone();
                let link = ctx.link().clone();

                spawn_local(async move {
                    match api.delete_post(id).await {
                        Ok(()) => link.send_message(Msg::PostDeleted(id)),
                        Err(e) => link.send_message(Msg::Error(e)),
                    }
                });

                false
            }

            Msg::PostDeleted(id) => {
                self.posts.retain(|p| p.id != id);
                self.posts_total -= 1;
                self.loading = false;
                true
            }

            // Cancel edit
            Msg::CancelEdit => {
                self.edit_state = EditState::None;
                self.edit_form_data = None;
                true
            }

            // Error
            Msg::Error(e) => {
                self.error = Some(e);
                self.loading = false;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let is_authenticated = self.token.is_some();

        html! {
            <div class="container">
                <h1>{ "Blog Application" }</h1>

                // Error display
                { self.view_error() }

                // Loading indicator
                { self.view_loading() }

                // Auth section
                if is_authenticated {
                    { self.view_user_info(ctx) }
                    { self.view_create_post_form(ctx) }
                } else {
                    { self.view_auth_forms(ctx) }
                }

                // Posts section
                { self.view_posts_section(ctx) }
            </div>
        }
    }
}

impl App {
    fn view_error(&self) -> Html {
        match &self.error {
            Some(error) => html! {
                <div class="error">
                    { format!("Error: {}", error) }
                </div>
            },
            None => html! {},
        }
    }

    fn view_loading(&self) -> Html {
        if self.loading {
            html! {
                <div class="loading"> { "Loading..." } </div>
            }
        } else {
            html! {}
        }
    }

    fn view_user_info(&self, ctx: &Context<Self>) -> Html {
        match &self.user {
            Some(user) => html! {
                <div class="user-info">
                    <span>{ format!("Logged in as: {} ({})", user.username, user.email) }</span>
                    <button onclick={ctx.link().callback(|_| Msg::Logout)}>
                        { "Logout" }
                    </button>
                </div>
            },
            None => html! {
                <div class="user-info">
                    <span>{ "Logged in" }</span>
                    <button onclick={ctx.link().callback(|_| Msg::Logout)}>
                        { "Logout" }
                    </button>
                </div>
            },
        }
    }

    fn view_auth_forms(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="auth-forms">
                <div class="register-form">
                    <h3>{ "Register" }</h3>
                    <input
                        type="text"
                        placeholder="Username"
                        value={self.register_username.clone()}
                        oninput={ctx.link().callback(|e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::UpdateRegisterUsername(input.value())
                        })}
                    />
                    <input
                        type="email"
                        placeholder="Email"
                        value={self.register_email.clone()}
                        oninput={ctx.link().callback(|e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::UpdateRegisterEmail(input.value())
                        })}
                    />
                    <input
                        type="password"
                        placeholder="Password"
                        value={self.register_password.clone()}
                        oninput={ctx.link().callback(|e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::UpdateRegisterPassword(input.value())
                        })}
                    />
                    <button onclick={ctx.link().callback(|_| Msg::Register)}>
                        { "Register" }
                    </button>
                </div>

                <div class="login-form">
                    <h3>{ "Login" }</h3>
                    <input
                        type="text"
                        placeholder="Username"
                        value={self.login_username.clone()}
                        oninput={ctx.link().callback(|e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::UpdateLoginUsername(input.value())
                        })}
                    />
                    <input
                        type="password"
                        placeholder="Password"
                        value={self.login_password.clone()}
                        oninput={ctx.link().callback(|e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::UpdateLoginPassword(input.value())
                        })}
                    />
                    <button onclick={ctx.link().callback(|_| Msg::Login)}>
                        { "Login" }
                    </button>
                </div>
            </div>
        }
    }

    fn view_create_post_form(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="create-post">
                <h3>{ "Create New Post" }</h3>
                <input
                    type="text"
                    placeholder="Title"
                    value={self.post_title.clone()}
                    oninput={ctx.link().callback(|e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Msg::UpdatePostTitle(input.value())
                    })}
                />
                <textarea
                    placeholder="Content"
                    value={self.post_content.clone()}
                    oninput={ctx.link().callback(|e: InputEvent| {
                        let input: HtmlTextAreaElement = e.target_unchecked_into();
                        Msg::UpdatePostContent(input.value())
                    })}
                />
                <button onclick={ctx.link().callback(|_| Msg::CreatePost)}>
                    { "Create Post" }
                </button>
            </div>
        }
    }

    fn view_posts_section(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="posts-section">
                <h2>{ "Posts" }</h2>
                <button onclick={ctx.link().callback(|_| Msg::LoadPosts)}>
                    { "Refresh Posts" }
                </button>

                <div class="posts-list">
                    { for self.posts.iter().map(|post| self.view_post(post, ctx)) }
                </div>

                if self.posts.is_empty() && !self.loading {
                    <p>{ "No posts yet. Be the first to create one!" }</p>
                }
            </div>
        }
    }

    fn view_post(&self, post: &Post, ctx: &Context<Self>) -> Html {
        let is_author = self
            .user
            .as_ref()
            .map(|u| u.id == post.author_id)
            .unwrap_or(false);
        let post_id = post.id;
        let post_title = post.title.clone();
        let post_content = post.content.clone();
        let post_author_id = post.author_id;
        let post_created_at = post.created_at.clone();

        match &self.edit_state {
            EditState::Editing { id, .. } if *id == post_id => {
                let (edit_title, edit_content) = match &self.edit_form_data {
                    Some((_, title, content)) => (title.clone(), content.clone()),
                    None => (post_title, post_content),
                };
                self.view_edit_form(post_id, edit_title, edit_content, ctx)
            }
            _ => {
                let edit_callback = { ctx.link().callback(move |_| Msg::EditPost(post_id)) };

                let delete_callback = { ctx.link().callback(move |_| Msg::DeletePost(post_id)) };

                html! {
                    <div class="post" key={post_id}>
                        <h3>{ &post_title }</h3>
                        <p>{ &post_content }</p>
                        <small>
                            { format!("By user {} at {}", post_author_id, post_created_at) }
                        </small>

                        if is_author {
                            <div class="post-actions">
                                <button onclick={edit_callback}>
                                    { "Edit" }
                                </button>
                                <button onclick={delete_callback}>
                                    { "Delete" }
                                </button>
                            </div>
                        }
                    </div>
                }
            }
        }
    }

    fn view_edit_form(&self, id: i64, title: String, content: String, ctx: &Context<Self>) -> Html {
        // Используем локальное состояние если есть
        let (current_title, current_content) = match &self.edit_form_data {
            Some((form_id, form_title, form_content)) if *form_id == id => {
                (form_title.clone(), form_content.clone())
            }
            _ => (title, content),
        };

        // Обработчик для поля заголовка - обновляем локальное состояние
        let title_handle = {
            let content = current_content.clone();
            ctx.link().callback(move |e: InputEvent| {
                let input: HtmlInputElement = e.target_unchecked_into();
                Msg::UpdatePost(id, input.value(), content.clone())
            })
        };

        // Обработчик для поля содержания
        let content_handle = {
            let title = current_title.clone();
            ctx.link().callback(move |e: InputEvent| {
                let input: HtmlTextAreaElement = e.target_unchecked_into();
                Msg::UpdatePost(id, title.clone(), input.value())
            })
        };

        // Обработчик для сохранения
        let save_handle = { ctx.link().callback(move |_| Msg::SavePost(id)) };

        // Обработчик для отмены
        let cancel_handle = ctx.link().callback(|_| Msg::CancelEdit);

        html! {
            <div class="edit-form">
                <h3>{ "Edit Post" }</h3>
                <input
                    type="text"
                    value={current_title}
                    oninput={title_handle}
                />
                <textarea
                    value={current_content}
                    oninput={content_handle}
                />
                <div class="edit-actions">
                    <button onclick={save_handle}>
                        { "Save" }
                    </button>
                    <button onclick={cancel_handle}>
                        { "Cancel" }
                    </button>
                </div>
            </div>
        }
    }
}

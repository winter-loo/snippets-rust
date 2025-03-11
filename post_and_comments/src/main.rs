#![allow(unused)]

use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    display_a_hacker_news_post();
    // display_a_pg_hacker_news_post();
}

fn display_a_pg_hacker_news_post() {
    let messages = [
        r#"2025-03-08 21:39:15 from Tom Lane <tgl(at)sss(dot)pgh(dot)pa(dot)us>"#,
        r#" 2025-03-09 01:15:25 from David Rowley <dgrowleyml(at)gmail(dot)com>"#,
        r#"  2025-03-09 02:31:57 from "David G(dot) Johnston" <david(dot)g(dot)johnston(at)gmail(dot)com>"#,
        r#" 2025-03-09 14:12:25 from Álvaro Herrera <alvherre(at)alvh(dot)no-ip(dot)org>"#,
        r#"  2025-03-09 15:45:41 from Tom Lane <tgl(at)sss(dot)pgh(dot)pa(dot)us>"#,
        r#"   2025-03-09 22:19:49 from Tom Lane <tgl(at)sss(dot)pgh(dot)pa(dot)us>"#,
        r#"    2025-03-10 00:14:49 from David Rowley <dgrowleyml(at)gmail(dot)com>"#,
        r#"     2025-03-10 01:13:28 from Tom Lane <tgl(at)sss(dot)pgh(dot)pa(dot)us>"#,
        r#"      2025-03-10 01:17:35 from David Rowley <dgrowleyml(at)gmail(dot)com>"#,
        r#"      2025-03-10 06:08:39 from Álvaro Herrera <alvherre(at)alvh(dot)no-ip(dot)org>"#,
        r#"       2025-03-10 16:18:29 from Tom Lane <tgl(at)sss(dot)pgh(dot)pa(dot)us>"#,
        r#"        2025-03-11 01:16:07 from David Rowley <dgrowleyml(at)gmail(dot)com>"#,
    ];

    let mut post = Post::new(&messages[0]);
    for msg in &messages[1..] {
        let n_leading_spaces = msg.chars().take_while(char::is_ascii_whitespace).count();
        if n_leading_spaces == 1 {
            post.add_comment(msg);
        } else {
            // if let Some(lastc) = post.last_comment_mut() {
            //     let mut com = lastc;
            //     for _ in 1..n_leading_spaces - 1 {
            //         if let Some(mut comment) = com.last_comment_mut() {
            //             com = comment;
            //         }
            //     }
            //     // com.add_comment(msg);
            // }
        }
    }
}

fn display_a_hacker_news_post() {
    // https://news.ycombinator.com/item?id=43280517
    let mut post = Post::new("Exploring Polymorphism in C: Lessons from Linux and FFmpeg's Code Design");
    let mut com1 = post.add_comment(r#"This is an excellent pattern in C. The Dovecot mail server has many fine examples of the style as well e.g.

    struct dict dict_driver_ldap = {
        .name = "ldap",
        .v = {
            .init = ldap_dict_init,
            .deinit = ldap_dict_deinit,
            .wait = ldap_dict_wait,
            .lookup = ldap_dict_lookup,
            .lookup_async = ldap_dict_lookup_async,
            .switch_ioloop = ldap_dict_switch_ioloop,
        }
    };

defines the virtual function table for the LDAP module, and any other subsystem that looks things up via the abstract dict interface can consequently be configured to use the ldap service without concrete knowledge of it.

(those interested in a deeper dive might start at https://github.com/dovecot/core/blob/main/src/lib-dict/dict-...)"#);

    let mut com1_1 = com1.add_comment(r#"So does the good old Quake 2 rendering API. The game exported a bunch of functions to the renderer via refimport_t and the renderer in return provided functions via refexport_t. The only visible symbol in a rendering DLL is GetRefAPI_t: https://github.com/id-Software/Quake-2/blob/master/client/re...

I remember being impressed by this approach, so I shamelessly copied it for my programming game: https://github.com/dividuum/infon/blob/master/renderer.h :)"#);

    let mut com1_1_1 = com1_1.add_comment(r#"I somehow suspect that the reason why Quake2 does this lies in the legacy of Quake1 written in DJGPP. DJGPP supports dynamicaly loaded libraries (although the API is technically unsupported and internal-only), but does not have any kind of dynamic linker, thus passing around pair of such structs during library initialization is the only way to make that work."#);
    let mut com1_1_2 = com1_1.add_comment(r#"Pretty sure Half-Life does something pretty similar - all functionality between the game and engine is done via function pointer structs."#);
    let mut com1_2 = com1.add_comment(r#"Reminds me of Apple’s CoreFoundation."#);
    let mut com2 = post.add_comment(r#"I spend a ton of time in FFmpeg, and I’m still blown away by how it uses abstractions to stay modular—especially for a project that’s been around forever and still feels so relevant. Those filtergraphs pulling off polymorphism-like tricks in C? It’s such an elegant way to manage complex pipelines. e.g.

ffmpeg -i input.wav -filter_complex " [0:a]asplit=2[a1][a2]; [a1]lowpass=f=500[a1_low]; [a2]highpass=f=500[a2_high]; [a1_low]volume=0.5[a1_low_vol]; [a2_high]volume=1.5[a2_high_vol]; [a1_low_vol][a2_high_vol]amix=inputs=2[a_mixed]; [a_mixed]aecho=0.8:0.9:1000:0.3[a_reverb] " -map "[a_reverb]" output.wav

That said, keeping those interfaces clean and consistent as the codebase grows (and ages) takes some real dedication.

Also recently joined the mailing lists and it’s been awesome to get a step closer to the pulse of the project. I recommend if you want to casually get more exposure to the breadth of the project.

https://ffmpeg.org/mailman/listinfo"#);
    let mut com2_1 = com2.add_comment(r#"how similar are the C abstractions in ffmpeg and qemu given they were started by the same person?"#);
    let mut com2_1_1 = com2_1.add_comment(r#"I haven’t worked with ffmpeg’s code, but I have worked with QEMU. QEMU has a lot of OOP (implemented in C obviously) that is supported by macros and GCC extensions. I definitely think it would have been better (and the code would be easier to work with) to use C++ rather than roll your own object model in C, but QEMU is quite old so it’s somewhat understandable. I say that as someone who mostly writes C and generally doesn’t like using C++."#);
    let mut com2_1_1_1 = com2_1_1.add_comment(r#"What's the reason for ffmpeg to use C, also historic?"#);
    let mut com2_1_1_1_1 = com2_1_1_1.add_comment(r#"C has less moving parts — it’s more difficult to define a subset of C++ that actually works across all platforms featuring a C++ compiler, not to mention of all the binary-incompatible versions of the C++ standard library that tend to exist — and C is supported on a wider variety of platforms. If you want to maximize portability, C is the way to go, and you run into much fewer problems."#);
    let mut com2_1_1_1_2 = com2_1_1_1.add_comment(r#"Much easier to link / load into other language binaries surely."#);
    
    println!("{post}");
}

struct CommentArena {
    comments: Vec<CommentInner>,
}

struct Comment {
    arena: Rc<RefCell<CommentArena>>,
    index: usize,
}

struct CommentInner {
    content: String,
    subs: Vec<Comment>,
    parent: Option<Comment>,
    prev: Option<Comment>,
    next: Option<Comment>,
}

struct Post {
    arena: Rc<RefCell<CommentArena>>,
}

impl std::fmt::Display for Post {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let comment = Comment {
            arena: Rc::clone(&self.arena),
            index: 0,
        };
        write!(f, "{}", comment)
    }
}

impl Post {
    fn new(content: &str) -> Self {
        let arena = Rc::new(RefCell::new(CommentArena { comments: vec![] }));
        let comment = CommentInner {
                    content: content.into(),
                    subs: vec![],
                    parent: None,
                    prev: None,
                    next: None,
                };
        arena.borrow_mut().comments.push(comment);
        Post {
            arena: Rc::clone(&arena),
        }
    }

    fn add_comment(&self, content: &str) -> Comment {
        let mut arena = self.arena.borrow_mut();
        let index = arena.comments.len();
        arena.comments.push(CommentInner {
            content: content.into(),
            subs: vec![],
            parent: Some(Comment {
                arena: Rc::clone(&self.arena),
                index: 0,
            }),
            prev: None,
            next: None,
        });

        Comment {
            arena: Rc::clone(&self.arena),
            index,
        }
    }

    // fn get_comment_mut(&mut self, index: usize) -> Option<&mut Comment> {
    //     self.comments.get_mut(index)
    // }

    // fn last_comment(&self) -> Option<&Comment> {
    //     self.comments.last()
    // }

    // fn last_comment_mut(&mut self) -> Option<&mut Comment> {
    //     self.comments.last_mut()
    // }
}

/// root | parent | prev | next
impl Comment {
    fn add_comment(&self, content: &str) -> Comment {
        let mut arena = self.arena.borrow_mut();
        let comments = &mut arena.comments;
        let index = comments.len();
        let inner = CommentInner {
            content: content.into(),
            subs: vec![],
            parent: None,
            prev: None,
            next: None,
        };
        comments.push(inner);

        Comment {
            arena: Rc::clone(&self.arena),
            index,
        }
    }

    // fn get_comment_mut(&mut self, index: usize) -> Option<&mut Comment> {
    //     self.subs.get_mut(index)
    // }

    // fn last_comment(&self) -> Option<&Comment> {
    //     self.subs.last()
    // }

    // fn last_comment_mut(&mut self) -> Option<&mut Comment> {
    //     self.subs.last_mut()
    // }

    fn root(&self) -> &Comment {
        todo!("...")
    }

    fn parent(&self) -> &Comment {
        todo!("..")
    }

    fn prev(&self) -> &Comment {
        todo!("..")
    }

    fn next(&self) -> &Comment {
        todo!("..")
    }

    fn print_content(content: &str, indents: &str, f: &mut std::fmt::Formatter<'_>) {
        for line in content.lines() {
            for i in 0.. {
                let start = i * 80;
                if start >= line.len() {
                    if i == 0 {
                        writeln!(f, "{indents}");
                    }
                    break;
                }
                let end = std::cmp::min(line.len(), i * 80 + 80);
                let print_line = &line[i * 80..end];
                writeln!(f, "{indents}{}", print_line);
            }
        }
    }

    fn strigify(&self, depth: u32, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let seperator: String = (0..40).map(|_| "-.").collect();
        let indents: String = (0..depth).map(|_| "----").collect();
        let arena = self.arena.borrow();
        let inner = &arena.comments[self.index];
        Self::print_content(&inner.content, &indents, f); 
        for com in &inner.subs {
            writeln!(f, "\n{}\n", seperator);
            com.strigify(depth + 1, f);
        }
        write!(f, "")
    }

}

impl std::fmt::Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.strigify(0, f)
    }
}

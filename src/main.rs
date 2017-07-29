extern crate lopdf;
use lopdf::{Document, Object, ObjectId};

enum Res {
    Ref(ObjectId),
    Embed,
    None
}

fn process_page(doc: &Document, dict: &lopdf::Dictionary) -> Res {
    use Object::Reference;
    match dict.get("Contents") {
        Some(contents) => {
            if let Reference(reference) = *contents {
                Res::Ref(reference)
            } else {
                Res::Embed
            }
        }
        _ => Res::None
    }
}

fn run() -> Result<(), std::io::Error> {
    let mut doc = Document::load("secret.pdf")?;
    let mut dict_to_pop = vec![];
    let mut array_to_pop = vec![];
    let mut obj_to_remove = vec![];
    for (page, c) in doc.get_pages() {
        // Find the mark
        if let Some(&Object::Dictionary(ref dict)) = doc.objects.get(&c) {
            match process_page(&doc, dict) {
                Res::Ref(oid) => array_to_pop.push(oid),
                Res::Embed => dict_to_pop.push(c),
                Res::None => ()
            }
        } else {
            unreachable!();
        };
        // Find the annotation
        if let Some(mut obj) = doc.objects.get_mut(&c) {
            if let Object::Dictionary(ref mut dict) = *obj {
                if let Some(obj) = dict.get("Annots") {
                    let to_remove = match *obj {
                        Object::Array(ref ary) => ary[0].clone(),
                        _ => obj.clone()
                    };
                    println!("{:?}", to_remove);
                    obj_to_remove.push(to_remove);
                }
                dict.remove("Annots");
            }
        }
    }
    for (oid, obj) in doc.objects.iter() {
        if let Object::Dictionary(ref dict) = *obj {
            if let Some(obj) = dict.get("A") {
                if let Object::Dictionary(_) = *obj {
                    obj_to_remove.push(Object::Reference(*oid));
                }
            }
        }
    }
    for oid in array_to_pop {
        if let Some(mut obj) = doc.objects.get_mut(&oid) {
            if let Object::Array(ref mut vect) = *obj {
                obj_to_remove.push(vect.pop().unwrap());
            }
        }
    }
    for oid in dict_to_pop {
        if let Some(mut obj) = doc.objects.get_mut(&oid) {
            if let Object::Dictionary(ref mut dict) = *obj {
                if let Some(mut array) = dict.get_mut("Contents") {
                    if let Object::Array(ref mut array) = *array {
                        obj_to_remove.push(array.pop().unwrap());
                    }
                }
            }
        }
    }
    for obj in obj_to_remove {
        if let Object::Reference(reference) = obj {
            doc.objects.remove(&reference);
        }
    }
    doc.save("modified.pdf");
    Ok(())
}

fn main() {
    run().unwrap();
}

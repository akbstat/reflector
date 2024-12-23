use lopdf::{Document, Object, ObjectId};
use std::{collections::BTreeMap, path::Path};

pub fn merge_pdf<P: AsRef<Path>>(files: &[P], dest: P) -> anyhow::Result<()> {
    let mut max_id = 1;
    let mut document_pages = BTreeMap::new();
    let mut document_objects = BTreeMap::new();
    let mut document = Document::with_version("1.7");
    // let mut toc_page_number = 0;
    for (_, file) in files.into_iter().enumerate() {
        let mut doc = Document::load(file)?;
        // if index.eq(&0) {
        //     toc_page_number = doc.get_pages().len();
        // }
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;
        document_pages.extend(
            doc.get_pages()
                .into_iter()
                .map(|(_, object_id)| (object_id, doc.get_object(object_id).unwrap().to_owned()))
                .collect::<BTreeMap<ObjectId, Object>>(),
        );
        document_objects.extend(doc.objects);
    }

    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    for (object_id, object) in document_objects.iter() {
        match object.type_name().unwrap_or("") {
            "Catalog" => {
                catalog_object = Some((
                    if let Some((id, _)) = catalog_object {
                        id
                    } else {
                        *object_id
                    },
                    object.clone(),
                ));
            }
            "Pages" => {
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, ref object)) = pages_object {
                        if let Ok(old_dictionary) = object.as_dict() {
                            dictionary.extend(old_dictionary);
                        }
                    }

                    pages_object = Some((
                        if let Some((id, _)) = pages_object {
                            id
                        } else {
                            *object_id
                        },
                        Object::Dictionary(dictionary),
                    ));
                }
            }
            "Page" => {}
            "Outlines" => {}
            "Outline" => {}
            _ => {
                document.objects.insert(*object_id, object.clone());
            }
        }
    }

    if pages_object.is_none() {
        println!("Pages root not found.");
        return Ok(());
    }
    for (object_id, object) in document_pages.iter() {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", pages_object.as_ref().unwrap().0);
            document
                .objects
                .insert(*object_id, Object::Dictionary(dictionary));
        }
    }
    if catalog_object.is_none() {
        println!("Catalog root not found.");
        return Ok(());
    }

    let catalog_object = catalog_object.unwrap();
    let pages_object = pages_object.unwrap();

    // Build a new "Pages" with updated fields
    if let Ok(dictionary) = pages_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Count", document_pages.len() as u32);
        dictionary.set(
            "Kids",
            document_pages
                .into_iter()
                .map(|(object_id, _)| Object::Reference(object_id))
                .collect::<Vec<_>>(),
        );

        document
            .objects
            .insert(pages_object.0, Object::Dictionary(dictionary));
    }

    // Build a new "Catalog" with updated fields
    if let Ok(dictionary) = catalog_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", pages_object.0);
        dictionary.remove(b"Outlines");
        document
            .objects
            .insert(catalog_object.0, Object::Dictionary(dictionary));
    }
    document.trailer.set("Root", catalog_object.0);
    document.max_id = document.objects.len() as u32;
    document.renumber_objects();
    document.adjust_zero_pages();
    if let Some(n) = document.build_outline() {
        if let Ok(x) = document.get_object_mut(catalog_object.0) {
            if let Object::Dictionary(ref mut dict) = x {
                dict.set("Outlines", Object::Reference(n));
            }
        }
    }

    document.compress();
    document.save(dest)?;
    Ok(())
}

// pub fn rebuild_toc_link<P>(p: P, db: &DBStruct, toc_pages: usize) -> anyhow::Result<()>
// where
//     P: AsRef<Path>,
// {
//     // build map
//     let mapper = db
//         .form
//         .iter()
//         .map(|f| (f.form.id, f.form.page))
//         .collect::<HashMap<usize, usize>>();

//     let mut document = Document::load(p.as_ref())?;
//     // add links into toc pages
//     let obj_ids = document
//         .objects
//         .iter()
//         .map(|(id, _)| id.clone())
//         .collect::<Vec<ObjectId>>();
//     for id in obj_ids {
//         let obj = document.get_object_mut(id)?;
//         if let Ok(obj) = obj.as_dict_mut() {
//             if obj.type_is(b"Annot") {
//                 if let Ok(dest) = obj.get(b"Dest") {
//                     if let Ok(dest) = dest.as_name_str() {
//                         let id = dest.to_string().parse::<usize>()?;
//                         if let Some(page) = mapper.get(&id) {
//                             obj.set(
//                                 b"Dest",
//                                 Object::Array(vec![
//                                     Object::Integer(toc_pages.add(page) as i64),
//                                     Object::Name(b"XYZ".into()),
//                                     Object::Null,
//                                     Object::Null,
//                                     Object::Null,
//                                     Object::Dictionary(dictionary! {
//                                         "XYZ" => vec![Object::Null, Object::Null, Object::Null]
//                                     }),
//                                 ]),
//                             );
//                         }
//                     }
//                 }
//             }
//         }
//     }
//     document.save(p.as_ref())?;
//     Ok(())
// }
